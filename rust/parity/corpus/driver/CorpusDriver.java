// SPDX-License-Identifier: Apache-2.0
//
// TAK-RS golden-corpus driver (parity harness, Phase 1a). Written for TAK-RS;
// it contains no code from src/. It calls only public APIs of the Java
// reference server's jars (CotParser, CotEventContainer,
// StreamingProtoBufHelper, DateUtil, NumericUtil, protobuf-java TextFormat)
// as black boxes and records what they return, so TAK-RS can be compared
// byte for byte with the reference (plan §1.4).
//
// Run only by rust/parity/harness.sh capture-corpus, inside the pinned JRE 17
// container with no network, a read-only root filesystem and the inputs
// mounted read-only:
//
//   java -cp <driver>:<reference jars> CorpusDriver <corpus-dir> <out-dir>
//
// Input and output formats: rust/parity/corpus/README.md.
// Exit status: 0 ok, 1 bad input or unexpected failure (message on stderr).
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SA-11 (Developer Testing and Evaluation), SR-4 (Provenance:
//   the runtime environment is recorded), SI-10 (inputs validated before use)

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.CharBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.Charset;
import java.nio.charset.CodingErrorAction;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.LinkOption;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.security.CodeSource;
import java.util.ArrayList;
import java.util.Date;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.TimeZone;
import java.util.TreeMap;
import java.util.regex.Pattern;

import javax.xml.parsers.SAXParserFactory;

import org.dom4j.Document;

import com.bbn.marti.remote.util.DateUtil;
import com.google.protobuf.TextFormat;

import atakmap.commoncommo.protobuf.v1.Takmessage.TakMessage;
import tak.server.cot.CotEventContainer;
import tak.server.cot.CotParser;
import tak.server.proto.StreamingProtoBufHelper;
import tak.server.util.NumericUtil;

public final class CorpusDriver {

    static final long MAX_INPUT_BYTES = 16L * 1024 * 1024;
    static final long MAX_TABLE_BYTES = 4L * 1024 * 1024;
    static final int MAX_ROWS = 20_000;
    static final int MAX_LINES = 100_000;

    static final Pattern ID = Pattern.compile("[A-Za-z0-9][A-Za-z0-9._-]{0,127}");
    static final Pattern INPUT =
        Pattern.compile("cases/(xml/[A-Za-z0-9][A-Za-z0-9._-]{0,127}\\.xml"
            + "|proto/[A-Za-z0-9][A-Za-z0-9._-]{0,127}\\.(txtpb|pb\\.hex)"
            + "|numbers/[A-Za-z0-9][A-Za-z0-9._-]{0,127}\\.(tsv|txt))");
    static final Pattern LONG = Pattern.compile("-?[0-9]{1,19}");
    static final Pattern BITS = Pattern.compile("(0[xX])?[0-9A-Fa-f]{16}");

    static final List<String> DIRECTIONS = List.of("x2p", "x2x", "p2x", "p2p");
    /** Number-table modes; each has exactly one MANIFEST row naming its input table. */
    static final List<String> TABLES = List.of("dbl-fmt", "dbl-parse", "int-parse", "time-fmt", "time-parse");

    // Two distinct defaults: a result equal to the default passed in both
    // calls means the reference fell back to its default (parse failure).
    static final double DBL_PROBE_A = 1.5;
    static final double DBL_PROBE_B = -2.25;
    static final int INT_PROBE_A = 7;
    static final int INT_PROBE_B = -9;

    /** Bad corpus input: the capture fails (exit 1), it is never a "reject". */
    static final class InputError extends Exception {
        private static final long serialVersionUID = 1L;

        InputError(String msg) {
            super(msg);
        }
    }

    /** One MANIFEST row; a null time means "-": the setter is not called. */
    record Row(int line, String id, String direction, String input, Long creationMs, Long submissionMs) {}

    record Outcome(byte[] bytes, String detail) {
        static Outcome ok(byte[] b) {
            return new Outcome(b, "");
        }

        static Outcome reject(String detail) {
            return new Outcome(null, detail);
        }

        boolean isOk() {
            return bytes != null;
        }
    }

    private final Path corpus;
    private final Path out;
    private final CotParser parser = new CotParser(false);

    private CorpusDriver(Path corpus, Path out) {
        this.corpus = corpus;
        this.out = out;
    }

    public static void main(String[] args) {
        if (args.length != 2) {
            System.err.println("usage: CorpusDriver <corpus-dir> <out-dir>");
            System.exit(1);
        }
        try {
            new CorpusDriver(Path.of(args[0]), Path.of(args[1])).run();
        } catch (InputError e) {
            System.err.println("corpus-driver: input error: " + e.getMessage());
            System.exit(1);
        } catch (Exception e) {
            System.err.println("corpus-driver: failed: " + e);
            e.printStackTrace(System.err);
            System.exit(1);
        }
        System.exit(0);
    }

    private void run() throws Exception {
        if (!Files.isDirectory(out, LinkOption.NOFOLLOW_LINKS)) {
            throw new InputError("output directory missing: " + out);
        }
        Map<String, String> tables = new TreeMap<>();
        List<Row> rows = readManifest(tables);
        StringBuilder results = new StringBuilder("id\tdirection\toutcome\toutput\tdetail\n");
        Map<String, Integer> counts = new TreeMap<>();
        for (Row r : rows) {
            Outcome o = runCase(r);
            String ext = r.direction().endsWith("2p") ? ".pb.hex" : ".xml";
            String rel = r.direction() + "/" + r.id() + ext;
            if (o.isOk()) {
                byte[] data = ext.equals(".xml") ? o.bytes() : hexLine(o.bytes());
                write(rel, data);
                results.append(r.id()).append('\t').append(r.direction()).append("\tok\t").append(rel).append("\t\n");
            } else {
                results.append(r.id()).append('\t').append(r.direction()).append("\treject\t-\t")
                    .append(o.detail()).append('\n');
            }
            counts.merge(r.direction() + (o.isOk() ? " ok" : " reject"), 1, Integer::sum);
        }
        write("results.tsv", results.toString().getBytes(StandardCharsets.UTF_8));
        numbers(tables);
        write("driver-env.json", environment().getBytes(StandardCharsets.UTF_8));
        for (Map.Entry<String, Integer> e : counts.entrySet()) {
            System.err.println("corpus-driver: " + e.getKey() + ": " + e.getValue());
        }
    }

    // -----------------------------------------------------------------------
    // Cases

    private Outcome runCase(Row r) throws InputError, IOException {
        switch (r.direction()) {
            case "x2p":
            case "x2x": {
                String xml = utf8(readInput(r.input()), r.input());
                return r.direction().equals("x2p") ? xmlToProto(xml, r) : xmlToXml(xml);
            }
            default: {
                byte[] pb = protoInput(r);
                return r.direction().equals("p2x") ? protoToXml(pb) : protoToProto(pb, r);
            }
        }
    }

    private Outcome xmlToProto(String xml, Row r) {
        try {
            Document doc = parser.parse(xml);
            if (doc == null) {
                return Outcome.reject("null");
            }
            CotEventContainer c = new CotEventContainer(doc);
            pinTimes(c, r);
            TakMessage m = StreamingProtoBufHelper.cot2protoBuf(c);
            return m == null ? Outcome.reject("null") : Outcome.ok(m.toByteArray());
        } catch (Exception | StackOverflowError e) {
            return Outcome.reject(e.getClass().getName());
        }
    }

    private Outcome xmlToXml(String xml) {
        try {
            Document doc = parser.parse(xml);
            if (doc == null) {
                return Outcome.reject("null");
            }
            return xmlBytes(new CotEventContainer(doc));
        } catch (Exception | StackOverflowError e) {
            return Outcome.reject(e.getClass().getName());
        }
    }

    private Outcome protoToXml(byte[] pb) {
        try {
            CotEventContainer c = StreamingProtoBufHelper.proto2cot(TakMessage.parseFrom(pb));
            return c == null ? Outcome.reject("null") : xmlBytes(c);
        } catch (Exception | StackOverflowError e) {
            return Outcome.reject(e.getClass().getName());
        }
    }

    private Outcome protoToProto(byte[] pb, Row r) {
        try {
            TakMessage in = TakMessage.parseFrom(pb);
            CotEventContainer c = StreamingProtoBufHelper.proto2cot(in);
            if (c == null) {
                return Outcome.reject("null");
            }
            // The container stamps "now" unless the input carried a creation
            // time; replace that wall-clock value with the manifest's.
            if (in.getCreationTime() <= 0 && r.creationMs() != null) {
                c.setCreationTime(r.creationMs());
            }
            TakMessage m = StreamingProtoBufHelper.cot2protoBuf(c);
            return m == null ? Outcome.reject("null") : Outcome.ok(m.toByteArray());
        } catch (Exception | StackOverflowError e) {
            return Outcome.reject(e.getClass().getName());
        }
    }

    private static void pinTimes(CotEventContainer c, Row r) {
        if (r.creationMs() != null) {
            c.setCreationTime(r.creationMs());
        }
        if (r.submissionMs() != null) {
            c.setSubmissionTime(new Date(r.submissionMs()));
        }
    }

    private static Outcome xmlBytes(CotEventContainer c) {
        String s = c.asXml();
        return s == null ? Outcome.reject("null") : Outcome.ok(s.getBytes(StandardCharsets.UTF_8));
    }

    /** Binary TakMessage input: .pb.hex as written, .txtpb encoded by protobuf-java. */
    private byte[] protoInput(Row r) throws InputError, IOException {
        byte[] raw = readInput(r.input());
        if (r.input().endsWith(".pb.hex")) {
            return parseHex(utf8(raw, r.input()), r.input());
        }
        TakMessage.Builder b = TakMessage.newBuilder();
        try {
            TextFormat.merge(utf8(raw, r.input()), b);
        } catch (TextFormat.ParseException e) {
            throw new InputError(r.input() + ": not valid TakMessage text format: " + e.getMessage());
        }
        byte[] pb = b.build().toByteArray();
        Path p = out.resolve("proto-inputs").resolve(r.id() + ".pb.hex");
        byte[] line = hexLine(pb);
        if (Files.exists(p, LinkOption.NOFOLLOW_LINKS)) {
            if (!java.util.Arrays.equals(Files.readAllBytes(p), line)) {
                throw new InputError(r.id() + ": text-format encoding differs between rows");
            }
        } else {
            write("proto-inputs/" + r.id() + ".pb.hex", line);
        }
        return pb;
    }

    // -----------------------------------------------------------------------
    // Number and time tables

    private void numbers(Map<String, String> tables) throws InputError, IOException {
        // dbl-fmt: Double.toString of each IEEE-754 bit pattern.
        List<String[]> dbl = doublesTable(tables.get("dbl-fmt"));
        StringBuilder sb = header();
        for (String[] d : dbl) {
            long bits = Long.parseUnsignedLong(d[1], 16);
            row(sb, d[0], d[1], "ok", Double.toString(Double.longBitsToDouble(bits)));
        }
        write("numbers/dbl-fmt.tsv", bytes(sb));

        sb = header();
        int n = 0;
        for (String s : lines(tables.get("dbl-parse"))) {
            n++;
            double a = NumericUtil.parseDoubleOrDefault(s, DBL_PROBE_A);
            double b = NumericUtil.parseDoubleOrDefault(s, DBL_PROBE_B);
            long ab = Double.doubleToRawLongBits(a);
            long bb = Double.doubleToRawLongBits(b);
            boolean dflt = ab == Double.doubleToRawLongBits(DBL_PROBE_A)
                && bb == Double.doubleToRawLongBits(DBL_PROBE_B);
            long v = ab == Double.doubleToRawLongBits(DBL_PROBE_A) ? bb : ab;
            String value = dflt ? "" : String.format(Locale.ROOT, "%016x", v);
            row(sb, Integer.toString(n), escape(s), dflt ? "default" : "ok", value);
        }
        write("numbers/dbl-parse.tsv", bytes(sb));

        sb = header();
        n = 0;
        for (String s : lines(tables.get("int-parse"))) {
            n++;
            int a = NumericUtil.parseIntOrDefault(s, INT_PROBE_A);
            int b = NumericUtil.parseIntOrDefault(s, INT_PROBE_B);
            boolean dflt = a == INT_PROBE_A && b == INT_PROBE_B;
            int v = a == INT_PROBE_A ? b : a;
            row(sb, Integer.toString(n), escape(s), dflt ? "default" : "ok", dflt ? "" : Integer.toString(v));
        }
        write("numbers/int-parse.tsv", bytes(sb));

        sb = header();
        n = 0;
        String timesMs = tables.get("time-fmt");
        for (String s : lines(timesMs)) {
            n++;
            if (!LONG.matcher(s).matches()) {
                throw new InputError(timesMs + ":" + n + ": not a decimal long: " + escape(s));
            }
            long ms;
            try {
                ms = Long.parseLong(s);
            } catch (NumberFormatException e) {
                throw new InputError(timesMs + ":" + n + ": out of long range");
            }
            try {
                String t = DateUtil.toCotTime(ms);
                row(sb, Integer.toString(n), s, t == null ? "reject" : "ok", t == null ? "null" : escape(t));
            } catch (Exception e) {
                row(sb, Integer.toString(n), s, "reject", e.getClass().getName());
            }
        }
        write("numbers/time-fmt.tsv", bytes(sb));

        sb = header();
        n = 0;
        for (String s : lines(tables.get("time-parse"))) {
            n++;
            try {
                row(sb, Integer.toString(n), escape(s), "ok", Long.toString(DateUtil.millisFromCotTimeStr(s)));
            } catch (Exception e) {
                row(sb, Integer.toString(n), escape(s), "reject", e.getClass().getName());
            }
        }
        write("numbers/time-parse.tsv", bytes(sb));
    }

    private static StringBuilder header() {
        return new StringBuilder("line\tinput\toutcome\tvalue\n");
    }

    private static void row(StringBuilder sb, String a, String b, String c, String d) {
        sb.append(a).append('\t').append(b).append('\t').append(c).append('\t').append(d).append('\n');
    }

    /** doubles.tsv rows as {line, bits (16 lowercase hex)}. */
    private List<String[]> doublesTable(String rel) throws InputError, IOException {
        List<String[]> rows = new ArrayList<>();
        int bitsCol = -1;
        int n = 0;
        for (String l : lines(rel)) {
            n++;
            if (l.isEmpty() || l.startsWith("#")) {
                continue;
            }
            String[] f = l.split("\t", -1);
            if (bitsCol < 0) {
                bitsCol = indexOf(f, "bits");
                if (bitsCol < 0) {
                    throw new InputError(rel + ":" + n + ": header row has no 'bits' column");
                }
                continue;
            }
            if (bitsCol >= f.length || !BITS.matcher(f[bitsCol]).matches()) {
                throw new InputError(rel + ":" + n + ": 'bits' must be 16 hex digits");
            }
            String hex = f[bitsCol].toLowerCase(Locale.ROOT);
            rows.add(new String[] {Integer.toString(n), hex.startsWith("0x") ? hex.substring(2) : hex});
        }
        if (bitsCol < 0) {
            throw new InputError(rel + ": no header row");
        }
        return rows;
    }

    // -----------------------------------------------------------------------
    // Manifest

    private List<Row> readManifest(Map<String, String> tables) throws InputError, IOException {
        String rel = "MANIFEST.tsv";
        String[] cols = null;
        int cId = -1, cDir = -1, cIn = -1, cCre = -1, cSub = -1;
        List<Row> rows = new ArrayList<>();
        Set<String> seen = new HashSet<>();
        Map<String, String> inputOf = new HashMap<>();
        int n = 0;
        for (String l : lines(rel)) {
            n++;
            if (l.isEmpty() || l.startsWith("#")) {
                continue;
            }
            String[] f = l.split("\t", -1);
            if (cols == null) {
                cols = f;
                cId = indexOf(f, "id");
                cDir = indexOf(f, "direction");
                cIn = indexOf(f, "input");
                cCre = indexOf(f, "creation_ms");
                cSub = indexOf(f, "submission_ms");
                if (cId < 0 || cDir < 0 || cIn < 0 || cCre < 0 || cSub < 0) {
                    throw new InputError(rel + ":" + n
                        + ": header must name the columns id, direction, input, creation_ms, submission_ms");
                }
                continue;
            }
            String where = rel + ":" + n + ": ";
            int need = Math.max(Math.max(Math.max(cId, cDir), Math.max(cIn, cCre)), cSub);
            if (f.length <= need) {
                throw new InputError(where + "expected at least " + (need + 1) + " columns");
            }
            String id = f[cId];
            String input = f[cIn];
            if (!ID.matcher(id).matches()) {
                throw new InputError(where + "invalid id '" + escape(id) + "'");
            }
            if (!INPUT.matcher(input).matches() || input.contains("..")) {
                throw new InputError(where + "invalid input path '" + escape(input) + "'");
            }
            String prev = inputOf.putIfAbsent(id, input);
            if (prev != null && !prev.equals(input)) {
                throw new InputError(where + "id " + id + " names two different inputs");
            }
            Long cre = millis(f[cCre], where + "creation_ms");
            Long sub = millis(f[cSub], where + "submission_ms");
            String d = f[cDir];
            String kind = input.substring("cases/".length(), input.indexOf('/', "cases/".length()));
            if (TABLES.contains(d)) {
                if (!kind.equals("numbers")) {
                    throw new InputError(where + "number mode " + d + " needs a cases/numbers/ input");
                }
                if (tables.putIfAbsent(d, input) != null) {
                    throw new InputError(where + "second row for number mode " + d);
                }
                continue;
            }
            if (!DIRECTIONS.contains(d)) {
                throw new InputError(where + "unknown direction '" + escape(d)
                    + "' (x2p, x2x, p2x, p2p, " + String.join(", ", TABLES) + ")");
            }
            if (!kind.equals(d.startsWith("x") ? "xml" : "proto")) {
                throw new InputError(where + "direction " + d + " does not fit input " + input);
            }
            if (!seen.add(id + "\t" + d)) {
                throw new InputError(where + "duplicate case " + id + " " + d);
            }
            rows.add(new Row(n, id, d, input, cre, sub));
            if (rows.size() > MAX_ROWS) {
                throw new InputError(rel + ": more than " + MAX_ROWS + " cases");
            }
        }
        if (cols == null) {
            throw new InputError(rel + ": no header row");
        }
        if (rows.isEmpty()) {
            throw new InputError(rel + ": no cases");
        }
        for (String t : TABLES) {
            if (!tables.containsKey(t)) {
                throw new InputError(rel + ": no row for number mode " + t);
            }
        }
        return rows;
    }

    /** A decimal i64, or "-" (null: the setter is not called). */
    private static Long millis(String v, String what) throws InputError {
        if (v.equals("-")) {
            return null;
        }
        if (!LONG.matcher(v).matches()) {
            throw new InputError(what + " is not a decimal integer: '" + escape(v) + "'");
        }
        try {
            return Long.parseLong(v);
        } catch (NumberFormatException e) {
            throw new InputError(what + " is out of range");
        }
    }

    private static int indexOf(String[] header, String name) {
        int found = -1;
        for (int i = 0; i < header.length; i++) {
            if (header[i].equals(name)) {
                if (found >= 0) {
                    return -1;
                }
                found = i;
            }
        }
        return found;
    }

    // -----------------------------------------------------------------------
    // I/O (bounded; no symlinks; inputs are UTF-8)

    private byte[] readInput(String rel) throws InputError, IOException {
        return readBounded(rel, MAX_INPUT_BYTES);
    }

    private byte[] readBounded(String rel, long max) throws InputError, IOException {
        Path p = corpus.resolve(rel).normalize();
        if (!p.startsWith(corpus.normalize())) {
            throw new InputError(rel + ": outside the corpus");
        }
        if (!Files.isRegularFile(p, LinkOption.NOFOLLOW_LINKS)) {
            throw new InputError(rel + ": missing, or not a regular file");
        }
        if (Files.size(p) > max) {
            throw new InputError(rel + ": larger than " + max + " bytes");
        }
        return Files.readAllBytes(p);
    }

    /** Lines of a table: split on LF only; a final LF ends the last line. */
    private List<String> lines(String rel) throws InputError, IOException {
        String s = utf8(readBounded(rel, MAX_TABLE_BYTES), rel);
        List<String> out = new ArrayList<>();
        int start = 0;
        while (start < s.length()) {
            int nl = s.indexOf('\n', start);
            if (nl < 0) {
                out.add(s.substring(start));
                break;
            }
            out.add(s.substring(start, nl));
            start = nl + 1;
            if (out.size() > MAX_LINES) {
                throw new InputError(rel + ": more than " + MAX_LINES + " lines");
            }
        }
        return out;
    }

    private static String utf8(byte[] b, String rel) throws InputError {
        try {
            CharBuffer cb = StandardCharsets.UTF_8.newDecoder()
                .onMalformedInput(CodingErrorAction.REPORT)
                .onUnmappableCharacter(CodingErrorAction.REPORT)
                .decode(ByteBuffer.wrap(b));
            return cb.toString();
        } catch (CharacterCodingException e) {
            throw new InputError(rel + ": not valid UTF-8 (use a .pb.hex input for invalid UTF-8 in proto fields)");
        }
    }

    /** Hex digits; ASCII whitespace ignored; '#' starts a comment to end of line. */
    private static byte[] parseHex(String s, String rel) throws InputError {
        StringBuilder h = new StringBuilder();
        boolean comment = false;
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            if (comment) {
                comment = c != '\n';
            } else if (c == '#') {
                comment = true;
            } else if (c == ' ' || c == '\t' || c == '\n' || c == '\r') {
                continue;
            } else if (Character.digit(c, 16) >= 0 && c < 0x80) {
                h.append(c);
            } else {
                throw new InputError(rel + ": invalid hex character at offset " + i);
            }
        }
        if (h.length() % 2 != 0) {
            throw new InputError(rel + ": odd number of hex digits");
        }
        byte[] b = new byte[h.length() / 2];
        for (int i = 0; i < b.length; i++) {
            b[i] = (byte) Integer.parseInt(h.substring(2 * i, 2 * i + 2), 16);
        }
        return b;
    }

    private static byte[] hexLine(byte[] b) {
        StringBuilder sb = new StringBuilder(b.length * 2 + 1);
        for (byte x : b) {
            sb.append(Character.forDigit((x >> 4) & 0xf, 16)).append(Character.forDigit(x & 0xf, 16));
        }
        return sb.append('\n').toString().getBytes(StandardCharsets.US_ASCII);
    }

    private static byte[] bytes(StringBuilder sb) {
        return sb.toString().getBytes(StandardCharsets.UTF_8);
    }

    private void write(String rel, byte[] data) throws IOException {
        Path p = out.resolve(rel);
        Files.createDirectories(p.getParent());
        Files.write(p, data, StandardOpenOption.CREATE_NEW, StandardOpenOption.WRITE, LinkOption.NOFOLLOW_LINKS);
    }

    /** TSV-safe, ASCII-only rendering: backslash escapes and \\uXXXX. */
    static String escape(String s) {
        StringBuilder sb = new StringBuilder(s.length());
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            switch (c) {
                case '\\' -> sb.append("\\\\");
                case '\t' -> sb.append("\\t");
                case '\n' -> sb.append("\\n");
                case '\r' -> sb.append("\\r");
                default -> {
                    if (c < 0x20 || c >= 0x7f) {
                        sb.append(String.format(Locale.ROOT, "\\u%04x", (int) c));
                    } else {
                        sb.append(c);
                    }
                }
            }
        }
        return sb.toString();
    }

    // -----------------------------------------------------------------------
    // Environment record (merged into PROVENANCE.json by the harness)

    private static String environment() {
        Map<String, String> m = new TreeMap<>();
        for (String k : new String[] {"java.version", "java.vendor", "java.runtime.version", "java.vm.name",
            "os.name", "os.arch", "file.encoding", "native.encoding", "sun.jnu.encoding", "user.language",
            "user.country"}) {
            m.put(k, String.valueOf(System.getProperty(k)));
        }
        m.put("default_charset", Charset.defaultCharset().name());
        m.put("default_timezone", TimeZone.getDefault().getID());
        m.put("default_locale", Locale.getDefault().toLanguageTag());
        String sax;
        try {
            sax = SAXParserFactory.newInstance().getClass().getName();
        } catch (RuntimeException | Error e) {
            sax = "error: " + e.getClass().getName();
        }
        m.put("sax_parser_factory", sax);
        Class<?>[] probes = {CotParser.class, CotEventContainer.class, StreamingProtoBufHelper.class,
            TakMessage.class, TextFormat.class, DateUtil.class, NumericUtil.class, Document.class};
        for (Class<?> c : probes) {
            m.put("class_source." + c.getName(), source(c));
        }
        for (String c : new String[] {"org.joda.time.DateTime", "org.apache.xerces.jaxp.SAXParserFactoryImpl"}) {
            try {
                m.put("class_source." + c, source(Class.forName(c, false, CorpusDriver.class.getClassLoader())));
            } catch (ClassNotFoundException e) {
                m.put("class_source." + c, "absent");
            }
        }
        List<String> jars = new ArrayList<>();
        for (String e : System.getProperty("java.class.path", "").split(java.io.File.pathSeparator)) {
            if (e.endsWith(".jar")) {
                jars.add(Path.of(e).getFileName().toString());
            }
        }
        jars.sort(null);
        m.put("classpath_jars", String.join(",", jars));
        StringBuilder sb = new StringBuilder("{\n");
        int i = 0;
        for (Map.Entry<String, String> e : m.entrySet()) {
            sb.append("  \"").append(json(e.getKey())).append("\": \"").append(json(e.getValue())).append('"')
                .append(++i < m.size() ? ",\n" : "\n");
        }
        return sb.append("}\n").toString();
    }

    private static String source(Class<?> c) {
        CodeSource cs = c.getProtectionDomain().getCodeSource();
        if (cs == null || cs.getLocation() == null) {
            return "bootstrap";
        }
        String loc = cs.getLocation().getPath();
        int slash = loc.lastIndexOf('/', loc.endsWith("/") ? loc.length() - 2 : loc.length());
        return slash >= 0 ? loc.substring(slash + 1) : loc;
    }

    private static String json(String s) {
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            if (c == '"' || c == '\\') {
                sb.append('\\').append(c);
            } else if (c < 0x20 || c >= 0x7f) {
                sb.append(String.format(Locale.ROOT, "\\u%04x", (int) c));
            } else {
                sb.append(c);
            }
        }
        return sb.toString();
    }
}
