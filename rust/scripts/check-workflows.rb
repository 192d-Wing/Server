# SPDX-License-Identifier: Apache-2.0
#
# Parsed-YAML half of scripts/check-workflows.sh (QA-006, SEC-004). Called
# only by that script; see its header for the rule list and the compliance
# mapping. Stdlib only (psych), no network, no gems:
#
#   ruby --disable=gems check-workflows.rb <repo-root> <workflow-file>...
#
# Every rule works on the decoded YAML node tree, so quoted keys (`"permissions":`),
# a space before the colon, escapes (`"rust\x2Dci-gate"`), block scalars
# (`name: >-`), plain scalars on the next line and flow style all resolve to
# the value GitHub sees. Scalars use YAML 1.2 core resolution for booleans and
# null (as GitHub does), so `on:` is the string key "on", not `true`.
#
# Output: one `FAIL <path>:<line>: <rule> <message>` line per violation, then
# `parsed <n> file(s)` as the last line. The shell wrapper requires that
# sentinel with the right count; anything else is a failure (fail closed).
# Exit: 0 no violation, 1 violation, 2 internal error.

require "psych"

repo = ARGV.shift or abort("usage: check-workflows.rb <repo-root> <file>...")
files = ARGV.dup
$violations = 0

NON_REQUIRED = %w[sbom annotations-strict].freeze # never in rust-ci-gate needs
# W5 (SEC-011): every `${{ ... }}` inside a run: value must be exactly one of
# these references. Anything else (github.event, inputs, needs, steps, index
# syntax github['event'], function calls such as format()/toJSON(), any case
# variant, env.*) fails. env.* is not allowed either: `${{ env.X }}` is pasted
# into the script text before the shell runs, so an env value taken from the
# event re-injects; use "$X" instead. Matrix values may not contain `${{`
# (W5b), so `${{ matrix.<key> }}` is a literal from the workflow file.
RUN_EXPR_OK = /\A(matrix\.[A-Za-z0-9_-]+|runner\.[A-Za-z0-9_]+|github\.(workspace|sha|run_id|run_attempt|repository))\z/
EXPR = /\$\{\{(.*?)\}\}/m
# SEC-010 (b): token permissions that can create or update a check run or
# commit status (spoofing the required rust-ci-gate check).
SPOOF_PERMS = %w[checks statuses].freeze
# SEC-012: key spellings GitHub accepts at the top, job, step and
# permissions level. A key whose lower case equals one of these but whose
# spelling differs fails (W8).
TOP_KEYS = %w[name run-name on permissions env defaults concurrency jobs].freeze
JOB_KEYS = %w[name permissions needs if runs-on environment concurrency outputs env defaults steps
              timeout-minutes strategy continue-on-error container services uses with secrets snapshot].freeze
STEP_KEYS = %w[id if name uses run working-directory shell with env continue-on-error timeout-minutes].freeze
PERM_KEYS = %w[actions artifact-metadata attestations checks contents deployments discussions id-token issues
               models packages pages pull-requests repository-projects security-events statuses].freeze
SHA_PIN = %r{\A[A-Za-z0-9_.-]+/[A-Za-z0-9_./-]+@[0-9a-f]{40}\z}
MAX_DEPTH = 64

class Doc
  attr_reader :rel, :lines

  def initialize(rel, text, allow_aliases)
    @rel = rel
    @lines = text.split("\n", -1)
    @allow_aliases = allow_aliases
    @anchors = {}
  end

  def fail(node, msg)
    line = node.respond_to?(:start_line) && node ? node.start_line + 1 : 0
    puts "FAIL #{@rel}:#{line}: #{msg}"
    $violations += 1
  end

  # Structural pass over the raw node tree (no alias resolution): records
  # anchors, rejects tags, complex keys and duplicate keys everywhere, and
  # (for rust-*.yml) anchors, aliases and merge keys.
  def scan(node, depth = 0)
    if depth > MAX_DEPTH
      fail(node, "W6 nesting deeper than #{MAX_DEPTH} levels")
      return
    end
    if node.respond_to?(:anchor) && node.anchor && !node.is_a?(Psych::Nodes::Alias)
      fail(node, "W6 YAML anchor &#{node.anchor} is not allowed in rust workflows") unless @allow_aliases
      @anchors[node.anchor] = node
    end
    if node.respond_to?(:tag) && node.tag
      fail(node, "W6 explicit YAML tag #{node.tag} is not allowed")
    end
    case node
    when Psych::Nodes::Alias
      fail(node, "W6 YAML alias *#{node.anchor} is not allowed in rust workflows") unless @allow_aliases
    when Psych::Nodes::Mapping
      seen = {}
      node.children.each_slice(2) do |k, v|
        if !k.is_a?(Psych::Nodes::Scalar)
          fail(k, "P2 mapping key is not a scalar (complex keys and aliased keys are not allowed)")
        else
          fail(k, "W6 merge key << is not allowed") if k.value == "<<" && k.plain
          if seen.key?(k.value)
            fail(k, "P3 duplicate key #{k.value.inspect} (first on line #{seen[k.value]}); parsers disagree on which wins")
          else
            seen[k.value] = k.start_line + 1
          end
        end
        scan(k, depth + 1)
        scan(v, depth + 1)
      end
    when Psych::Nodes::Sequence
      node.children.each { |c| scan(c, depth + 1) }
    end
  end

  def res(node)
    return node unless node.is_a?(Psych::Nodes::Alias)
    @anchors[node.anchor]
  end

  # [key-string, key-node, value-node] for a mapping (aliases resolved).
  def pairs(map)
    map = res(map)
    return [] unless map.is_a?(Psych::Nodes::Mapping)
    map.children.each_slice(2).map do |k, v|
      [k.is_a?(Psych::Nodes::Scalar) ? k.value : nil, k, res(v)]
    end
  end

  def get(map, name)
    p = pairs(map).find { |k, _, _| k == name }
    p && p[2]
  end

  def mapping?(n)
    res(n).is_a?(Psych::Nodes::Mapping)
  end

  # YAML 1.2 core schema for plain scalars: null and booleans; the rest are
  # strings (numbers are compared as text, which is all the rules need).
  def sval(n)
    n = res(n)
    return :non_scalar unless n.is_a?(Psych::Nodes::Scalar)
    return n.value unless n.plain
    case n.value
    when "", "~", "null", "Null", "NULL" then nil
    when "true", "True", "TRUE" then true
    when "false", "False", "FALSE" then false
    else n.value
    end
  end

  def plain_data(n, depth = 0)
    n = res(n)
    return :too_deep if depth > MAX_DEPTH
    case n
    when Psych::Nodes::Mapping then pairs(n).map { |k, _, v| [k, plain_data(v, depth + 1)] }.to_h
    when Psych::Nodes::Sequence then n.children.map { |c| plain_data(c, depth + 1) }
    when Psych::Nodes::Scalar then sval(n)
    end
  end

  # Every scalar string under n (for matrix values).
  def strings(n, depth = 0, out = [])
    n = res(n)
    return out if depth > MAX_DEPTH
    case n
    when Psych::Nodes::Mapping then pairs(n).each { |k, _, v| out << k.to_s; strings(v, depth + 1, out) }
    when Psych::Nodes::Sequence then n.children.each { |c| strings(c, depth + 1, out) }
    when Psych::Nodes::Scalar then out << n.value
    end
    out
  end

  # Yields [path-array, key-string, key-node, value-node] for every mapping
  # entry under n.
  def each_entry(n, path = [], depth = 0, &blk)
    n = res(n)
    return if depth > MAX_DEPTH
    case n
    when Psych::Nodes::Mapping
      pairs(n).each do |k, kn, v|
        blk.call(path, k, kn, v)
        each_entry(v, path + [k], depth + 1, &blk)
      end
    when Psych::Nodes::Sequence
      n.children.each_with_index { |c, i| each_entry(c, path + [i], depth + 1, &blk) }
    end
  end
end

def perms_ok?(doc, node)
  doc.plain_data(node) == { "contents" => "read" }
end

def trigger_names(doc, on)
  case (n = doc.res(on))
  when Psych::Nodes::Scalar then [n.value]
  when Psych::Nodes::Sequence then n.children.map { |c| doc.sval(c).to_s }
  when Psych::Nodes::Mapping then doc.pairs(n).map { |k, _, _| k.to_s }
  else []
  end
end

# Expressions in a run: string that are not allow-listed (an unterminated
# `${{` counts as one).
def run_exprs_bad(s)
  bad = s.scan(EXPR).map { |(e)| e.strip }.reject { |e| e =~ RUN_EXPR_OK }
  bad << "${{ (unterminated)" if s.gsub(EXPR, "").include?("${{")
  bad
end

# SEC-010 (b) and W9, for EVERY workflow file: top-level permissions: is
# present (without it the token gets the repository default, which may be
# write to checks/statuses), and neither the top level nor any job grants
# write-all, checks: write or statuses: write.
def check_token(doc, root)
  top = doc.get(root, "permissions")
  doc.fail(root, "W9 top-level permissions: missing (the token would get the repository default)") if top.nil?
  nodes = [[top, "top level"]]
  jobs = doc.get(root, "jobs")
  doc.pairs(jobs).each { |id, _, job| nodes << [doc.get(job, "permissions"), "job #{id}"] if doc.mapping?(job) } if doc.mapping?(jobs)
  nodes.each do |n, where|
    next if n.nil?
    v = doc.sval(n)
    if v.is_a?(String)
      doc.fail(n, "W9 #{where}: permissions: #{v} is not allowed") if v.strip.downcase == "write-all"
    elsif doc.mapping?(n)
      doc.pairs(n).each do |k, kn, pv|
        next unless k && SPOOF_PERMS.include?(k.downcase)
        pvs = doc.sval(pv)
        next if pvs.is_a?(String) && %w[read none].include?(pvs.strip.downcase)
        doc.fail(kn, "W9 #{where}: permissions #{k}: #{pvs.inspect} is not allowed (a token that writes check runs or statuses can spoof rust-ci-gate)")
      end
    end
  end
end

# SEC-012 (W8): a structural key whose lower case is a known key but whose
# spelling differs (`Name:`, `Uses:`, `Permissions:`).
def check_key_case(doc, root)
  chk = lambda do |map, known, where|
    doc.pairs(map).each do |k, kn, _|
      next unless k.is_a?(String)
      kk = known.find { |x| x.downcase == k.downcase }
      doc.fail(kn, "W8 #{where}: key #{k.inspect} must be spelled #{kk.inspect}") if kk && kk != k
    end
  end
  chk.call(root, TOP_KEYS, "top level")
  perm = doc.get(root, "permissions")
  chk.call(perm, PERM_KEYS, "permissions") if doc.mapping?(perm)
  jobs = doc.get(root, "jobs")
  return unless doc.mapping?(jobs)
  doc.pairs(jobs).each do |id, _, job|
    next unless doc.mapping?(job)
    chk.call(job, JOB_KEYS, "job #{id}")
    jp = doc.get(job, "permissions")
    chk.call(jp, PERM_KEYS, "job #{id} permissions") if doc.mapping?(jp)
    steps = doc.res(doc.get(job, "steps"))
    next unless steps.is_a?(Psych::Nodes::Sequence)
    steps.children.each { |st| chk.call(st, STEP_KEYS, "job #{id} step") if doc.mapping?(st) }
  end
end

def check_rust(doc, root, is_ci)
  # W2: forbidden triggers (parsed) and any mention outside comments (text).
  on = doc.get(root, "on")
  trigger_names(doc, on).each do |t|
    doc.fail(on, "W2 #{t} trigger is forbidden") if %w[pull_request_target workflow_run].include?(t)
  end
  doc.lines.each_with_index do |l, i|
    next if l =~ /\A\s*#/
    if l =~ /(^|[^A-Za-z_])(pull_request_target|workflow_run)([^A-Za-z_]|$)/
      puts "FAIL #{doc.rel}:#{i + 1}: W2 pull_request_target/workflow_run is forbidden"
      $violations += 1
    end
  end

  # W3: top-level permissions exactly contents: read.
  perm = doc.get(root, "permissions")
  if perm.nil?
    doc.fail(root, "W3 top-level permissions: missing")
  elsif !perms_ok?(doc, perm)
    doc.fail(perm, "W3 top-level permissions must be exactly contents: read (got #{doc.plain_data(perm).inspect})")
  end

  jobs = doc.get(root, "jobs")
  unless doc.mapping?(jobs)
    doc.fail(root, "J1 jobs: missing or not a mapping")
    return
  end

  # W7 (any depth, any key spelling) and W5 (every run: string).
  doc.each_entry(root) do |path, k, kn, v|
    next if k.nil?
    if k.downcase == "continue-on-error"
      allowed = is_ci && path == ["jobs", "sbom"] && k == "continue-on-error"
      doc.fail(kn, "W7 continue-on-error is allowed only as the job-level key of job sbom in rust-ci.yml (at #{(path + [k]).join('.')})") unless allowed
    end
    if k.downcase == "run" && (s = doc.sval(v)).is_a?(String)
      run_exprs_bad(s).each do |e|
        doc.fail(v, "W5 template expression #{e.inspect} inside run: is not allowed (only matrix.<key>, runner.*, github.workspace/sha/run_id/run_attempt/repository; pass other values through env: and use \"$VAR\")")
      end
    end
  end

  doc.pairs(jobs).each do |id, idn, job|
    unless doc.mapping?(job)
      doc.fail(idn, "J2 job #{id.inspect} is not a mapping")
      next
    end
    # W5b (SEC-010 (a), SEC-011): no expression anywhere in strategy:, so
    # matrix values are literals (they feed job names and ${{ matrix.* }}).
    if (strat = doc.get(job, "strategy"))
      exprs = doc.strings(strat).select { |x| x.include?("${{") }
      doc.fail(strat, "W5b job #{id}: strategy/matrix must not contain ${{ }} expressions: #{exprs.inspect}") unless exprs.empty?
    end
    # W3b
    jp = doc.get(job, "permissions")
    doc.pairs(job).each do |k, kn, _|
      next unless k && k != "permissions" && k.downcase == "permissions"
      doc.fail(kn, "W3b job #{id}: key #{k.inspect} is not allowed")
    end
    if jp && !perms_ok?(doc, jp)
      doc.fail(jp, "W3b job #{id}: job-level permissions must be absent or exactly contents: read (got #{doc.plain_data(jp).inspect})")
    end
    # W1 / W4
    uses_nodes = []
    ju = doc.get(job, "uses")
    uses_nodes << [ju, nil] if ju
    steps = doc.get(job, "steps")
    if steps
      if !doc.res(steps).is_a?(Psych::Nodes::Sequence)
        doc.fail(steps, "J3 job #{id}: steps is not a list")
      else
        doc.res(steps).children.each do |st|
          st = doc.res(st)
          next unless st.is_a?(Psych::Nodes::Mapping)
          su = doc.get(st, "uses")
          uses_nodes << [su, st] if su
        end
      end
    end
    uses_nodes.each do |un, st|
      u = doc.sval(un)
      unless u.is_a?(String)
        doc.fail(un, "W1 uses: is not a string")
        next
      end
      if u.start_with?("./")
        next
      elsif u !~ SHA_PIN
        doc.fail(un, "W1 uses: not pinned to a 40-hex commit SHA: #{u}")
        next
      end
      rest = doc.lines[un.end_line].to_s[un.end_column..-1].to_s
      doc.fail(un, "W1 uses: SHA pin without a '# v<version>' comment on the same line: #{u}") unless rest =~ /\A\s*#\s*v[0-9]/
      if u.start_with?("actions/checkout@")
        pc = st && doc.get(doc.get(st, "with") || Psych::Nodes::Mapping.new, "persist-credentials")
        pv = pc && doc.sval(pc)
        doc.fail(un, "W4 actions/checkout without persist-credentials: false") unless [false, "false"].include?(pv)
      end
    end
  end

  check_ci(doc, root, on, jobs) if is_ci
end

def check_ci(doc, root, on, jobs)
  # B1: on.pull_request without path filters.
  if !trigger_names(doc, on).include?("pull_request")
    doc.fail(on || root, "B1 rust-ci.yml has no on.pull_request trigger")
  elsif doc.mapping?(on) && (pr = doc.get(on, "pull_request")) && doc.mapping?(pr)
    %w[paths paths-ignore].each do |f|
      doc.fail(doc.get(pr, f), "B1 rust-ci.yml on.pull_request must not have #{f}") if doc.get(pr, f)
    end
  end

  ids = doc.pairs(jobs).map { |k, _, _| k.to_s }
  gate = doc.get(jobs, "rust-ci-gate")
  unless gate && doc.mapping?(gate)
    doc.fail(jobs, "B3 rust-ci.yml defines no job rust-ci-gate")
    return
  end

  # B2 inside rust-ci.yml: only the gate may look like the gate.
  doc.pairs(jobs).each do |id, idn, job|
    name = doc.mapping?(job) ? doc.get(job, "name") : nil
    nv = name ? doc.sval(name) : nil
    nv = nv.to_s unless nv.nil?
    if id == "rust-ci-gate"
      doc.fail(name, "B2 rust-ci-gate name must be absent or exactly rust-ci-gate (got #{nv.inspect})") if name && nv != "rust-ci-gate"
      %w[strategy uses].each { |k| doc.fail(doc.get(job, k), "B3 rust-ci-gate must not have #{k}:") if doc.get(job, k) }
      next
    end
    doc.fail(idn, "B2 job ID #{id.inspect} contains rust-ci (reserved for the gate)") if id.to_s =~ /rust-ci/i
    next unless nv
    doc.fail(name, "B2 job #{id} name contains rust-ci: #{nv.inspect}") if nv =~ /rust-ci/i
    if nv.include?("${{")
      only_matrix = nv.scan(/\$\{\{(.*?)\}\}/m).all? { |(e)| e =~ /\A\s*matrix\.[A-Za-z0-9_-]+\s*\z/ } && nv !~ /\$\{\{(?!.*?\}\})/m
      strat = doc.get(job, "strategy")
      bad_vals = strat ? doc.strings(strat).select { |x| x =~ /rust-ci/i || x.include?("${{") } : []
      if !only_matrix
        doc.fail(name, "B2 job #{id} name may use only ${{ matrix.<key> }} expressions: #{nv.inspect}")
      elsif !bad_vals.empty?
        doc.fail(name, "B2 job #{id} matrix values contain rust-ci or ${{ }}: #{bad_vals.inspect}")
      end
    end
  end

  # B3: if: always(); needs = all job IDs minus the gate and NON_REQUIRED.
  iv = doc.sval(doc.get(gate, "if") || Psych::Nodes::Scalar.new(""))
  unless iv.is_a?(String) && iv.strip =~ /\A(always\(\)|\$\{\{\s*always\(\)\s*\}\})\z/
    doc.fail(gate, "B3 rust-ci-gate must have 'if: always()' (got #{iv.inspect})")
  end
  nd = doc.plain_data(doc.get(gate, "needs") || Psych::Nodes::Sequence.new)
  nd = [nd] if nd.is_a?(String)
  if !nd.is_a?(Array) || !nd.all? { |x| x.is_a?(String) }
    doc.fail(gate, "B3 rust-ci-gate needs: must be a list of job IDs")
  else
    want = (ids - ["rust-ci-gate"] - NON_REQUIRED).sort
    got = nd.sort
    unless got == want && nd.uniq.size == nd.size
      doc.fail(doc.get(gate, "needs") || gate,
               "B3 rust-ci-gate needs [#{got.join(',')}] != jobs minus rust-ci-gate/#{NON_REQUIRED.join('/')} [#{want.join(',')}]")
    end
  end
end

# B2 for every workflow other than rust-ci.yml.
def check_other(doc, root)
  jobs = doc.get(root, "jobs")
  return if jobs.nil?
  unless doc.mapping?(jobs)
    doc.fail(jobs, "B2 jobs: is not a mapping")
    return
  end
  doc.pairs(jobs).each do |id, idn, job|
    if id == "<<"
      doc.fail(idn, "B2 merge key << under jobs: is not allowed")
      next
    end
    doc.fail(idn, "B2 job ID #{id.inspect} contains rust-ci (only rust-ci.yml may define rust-ci jobs)") if id.to_s =~ /rust-ci/i
    next unless doc.mapping?(job)
    doc.pairs(job).each { |k, kn, _| doc.fail(kn, "B2 merge key << in job #{id} is not allowed") if k == "<<" }
    name = doc.get(job, "name")
    next unless name
    nv = doc.sval(name)
    if nv == :non_scalar
      doc.fail(name, "B2 job #{id} name is not a scalar")
      next
    end
    nv = nv.to_s
    doc.fail(name, "B2 job #{id} name contains rust-ci: #{nv.inspect}") if nv =~ /rust-ci/i
    doc.fail(name, "B2 job #{id} name contains an expression: #{nv.inspect}") if nv.include?("${{")
  end
end

parsed = 0
begin
  files.each do |f|
    rel = f.start_with?(repo + "/") ? f[(repo.length + 1)..-1] : f
    base = File.basename(f)
    is_rust = base =~ /\Arust-.*\.ya?ml\z/
    is_ci = base == "rust-ci.yml"
    text = File.binread(f).force_encoding("UTF-8")
    doc = Doc.new(rel, text, !is_rust)
    unless text.valid_encoding?
      doc.fail(nil, "P0 not valid UTF-8")
      parsed += 1
      next
    end
    begin
      stream = Psych.parse_stream(text, filename: rel)
    rescue Psych::SyntaxError => e
      doc.fail(nil, "P0 not valid YAML (#{e.message.lines.first.strip})")
      parsed += 1
      next
    end
    if stream.children.size != 1
      doc.fail(nil, "P1 expected exactly one YAML document (got #{stream.children.size})")
      parsed += 1
      next
    end
    root = stream.children[0].root
    doc.scan(root)
    if !root.is_a?(Psych::Nodes::Mapping)
      doc.fail(root, "P1 top level is not a mapping")
    else
      check_token(doc, root)
      check_key_case(doc, root)
      if is_rust
        check_rust(doc, root, is_ci)
        check_other(doc, root) unless is_ci
      else
        check_other(doc, root)
      end
    end
    parsed += 1
  end
rescue StandardError => e
  puts "ERROR internal: #{e.class}: #{e.message}"
  exit 2
end
puts "parsed #{parsed} file(s)"
exit($violations.zero? ? 0 : 1)
