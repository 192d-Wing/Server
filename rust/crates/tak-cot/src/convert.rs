// SPDX-License-Identifier: Apache-2.0

//! `CoT` XML <-> TAK Protocol v1 conversion: [`to_tak_message`] and
//! [`from_tak_message`], including the `<detail>` split into typed
//! sub-messages plus `xmlDetail` (parity-map sections A and B).
//!
//! Both directions reproduce what the reference server puts on the wire,
//! including behaviour that looks accidental (the "Kept quirk" rows of the
//! parity map): typed detail elements are recognised only with an exact
//! attribute set, only the first element of each name is considered,
//! unreadable numbers fall back to fixed defaults, unsigned fields are shown
//! as Java signed values, and an `xmlDetail` element overrides the typed
//! message of the same name.
//!
//! Every input is untrusted. Nothing here panics or recurses; the nested
//! `xmlDetail` XML is parsed with the event's own [`Limits`], and the
//! `xmlDetail` text built for a TAK Protocol client is capped by
//! `Limits::max_xml_detail_bytes`.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SI-10 (Information Input Validation), SC-5 (Denial-of-service
//   Protection)

use std::collections::VecDeque;

use tak_proto::v1::{
    Contact, CotEvent as ProtoEvent, Detail, Group, PrecisionLocation, Status, TakMessage, Takv,
    Track,
};

use crate::error::{ConvertError, LimitKind, TimeField, XmlError, XmlErrorKind};
use crate::event::{CotEvent, NumericCache};
use crate::javafmt::{
    cot_time_to_string, double_to_string, parse_cot_time, parse_double, parse_int,
};
use crate::limits::Limits;
use crate::xml::{Document, ElementRef, NodeId};

/// Values the caller supplies for an XML-to-proto conversion.
///
/// Each `Some` replaces the event's own value ([`CotEvent::creation_ms`],
/// [`CotEvent::submission_ms`]); `None` keeps it. Only a positive result is
/// written to the message (parity-map K-11).
///
/// `ToProtoOptions::default()` writes no `creationTime` for an event parsed
/// from XML (it has none), whereas the reference server always writes one:
/// the time it received the event. A server path that forwards events to TAK
/// Protocol clients must set `creation_ms` to the receipt time (Phase 3);
/// the default suits only tests and offline conversion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ToProtoOptions {
    /// Creation time in milliseconds since the epoch. The reference stamps
    /// "now" when an event is received; TAK-RS takes it from the caller.
    pub creation_ms: Option<i64>,
    /// Submission time in milliseconds since the epoch.
    pub submission_ms: Option<i64>,
}

/// The six `detail` children that have a typed TAK Protocol message, in the
/// order the proto-to-XML direction writes them (K-18).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Typed {
    Contact,
    Group,
    PrecisionLocation,
    Status,
    Takv,
    Track,
}

impl Typed {
    const ALL: [Self; 6] = [
        Self::Contact,
        Self::Group,
        Self::PrecisionLocation,
        Self::Status,
        Self::Takv,
        Self::Track,
    ];

    const fn element_name(self) -> &'static str {
        match self {
            Self::Contact => "contact",
            Self::Group => "__group",
            Self::PrecisionLocation => "precisionlocation",
            Self::Status => "status",
            Self::Takv => "takv",
            Self::Track => "track",
        }
    }

    /// Fills this kind's message in `out` from `el` when `el` has exactly
    /// the attribute set the kind requires (K-06, K-07, K-08); returns
    /// whether it did, i.e. whether `el` leaves `xmlDetail`.
    fn extract(self, el: ElementRef<'_>, cache: &NumericCache, out: &mut Detail) -> bool {
        match self {
            Self::Contact => {
                // `callsign` is required and `endpoint` optional; nothing
                // else may be present. An empty endpoint is dropped.
                let Some(callsign) = el.attribute("callsign") else {
                    return false;
                };
                let endpoint = el.attribute("endpoint");
                let expected = if endpoint.is_some() { 2 } else { 1 };
                if el.attributes().len() != expected {
                    return false;
                }
                out.contact = Some(Contact {
                    endpoint: endpoint.unwrap_or_default().to_owned(),
                    callsign: callsign.to_owned(),
                });
            }
            Self::Group => {
                let Some([name, role]) = exact_attributes(el, ["name", "role"]) else {
                    return false;
                };
                out.group = Some(Group {
                    name: name.to_owned(),
                    role: role.to_owned(),
                });
            }
            Self::PrecisionLocation => {
                let Some([geopointsrc, altsrc]) = exact_attributes(el, ["geopointsrc", "altsrc"])
                else {
                    return false;
                };
                out.precision_location = Some(PrecisionLocation {
                    geopointsrc: geopointsrc.to_owned(),
                    altsrc: altsrc.to_owned(),
                });
            }
            Self::Status => {
                let Some([battery]) = exact_attributes(el, ["battery"]) else {
                    return false;
                };
                let battery = NumericCache::int(cache.battery)
                    .unwrap_or_else(|| parse_int(battery).unwrap_or(0));
                out.status = Some(Status {
                    battery: battery.cast_unsigned(),
                });
            }
            Self::Takv => {
                let Some([device, platform, os, version]) =
                    exact_attributes(el, ["device", "platform", "os", "version"])
                else {
                    return false;
                };
                out.takv = Some(Takv {
                    device: device.to_owned(),
                    platform: platform.to_owned(),
                    os: os.to_owned(),
                    version: version.to_owned(),
                });
            }
            Self::Track => {
                let Some([speed, course]) = exact_attributes(el, ["speed", "course"]) else {
                    return false;
                };
                out.track = Some(Track {
                    speed: NumericCache::double(cache.speed)
                        .unwrap_or_else(|| parse_double(speed).unwrap_or(0.0)),
                    course: NumericCache::double(cache.course)
                        .unwrap_or_else(|| parse_double(course).unwrap_or(0.0)),
                });
            }
        }
        true
    }
}

/// The values of `names` on `el` (matched by local name), when `el` has
/// exactly that many attributes and each name is among them.
fn exact_attributes<'a, const N: usize>(
    el: ElementRef<'a>,
    names: [&str; N],
) -> Option<[&'a str; N]> {
    if el.attributes().len() != N {
        return None;
    }
    let mut values = [""; N];
    for (slot, name) in values.iter_mut().zip(names) {
        *slot = el.attribute(name)?;
    }
    Some(values)
}

/// Converts a `CoT` event to the TAK Protocol message the reference server
/// sends to protocol clients.
///
/// - `type`, `uid`, `how`, `caveat`, `releasableTo` (proto
///   `releaseableTo`), `opex`, `qos` and `access` are copied from the root
///   element when present (K-01, K-04);
/// - `time`, `start` and `stale` become milliseconds, preferring the values
///   remembered from a proto-built event (K-16); a missing attribute gives
///   0 and an unreadable one fails the conversion (K-02, DEV-05);
/// - the first `point` gives lat, lon, hae (unreadable: 0) and ce, le
///   (unreadable: 999999); no `point` gives all zeros (K-03);
/// - a `detail` element always produces a `Detail` (K-05): the first
///   `contact`, `__group`, `precisionlocation`, `status`, `takv` and `track`
///   become typed messages when their attribute sets are exact (K-06 to
///   K-09), and every other child element is serialized, in order, into
///   `xmlDetail` (K-10);
/// - creation and submission times from `opts` or the event, when positive
///   (K-11). The reference always writes `creationTime` (its receipt time);
///   for the same output a caller sets [`ToProtoOptions::creation_ms`] to
///   the time the event was received, as nothing here reads a clock.
///
/// # Errors
///
/// [`ConvertError::Time`] for an unreadable time attribute;
/// [`ConvertError::XmlDetailTooLarge`] when `xmlDetail` would exceed
/// `Limits::max_xml_detail_bytes` of the event's document.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SI-10 (Information Input Validation), SI-10(3) (Predictable
//   Behavior), SC-5 (Denial-of-service Protection)
// - ASD STIG: APSC-DV-002530 / V-222606 (unverified)
// PARITY: src/takserver-plugins/src/main/java/tak/server/proto/StreamingProtoBufHelper.java#cot2protoBuf
pub fn to_tak_message(ev: &CotEvent, opts: &ToProtoOptions) -> Result<TakMessage, ConvertError> {
    let doc = ev.document();
    let root = doc.root();
    let cache = &ev.cache;
    let copy = |name: &str| root.attribute(name).map(str::to_owned).unwrap_or_default();

    let mut out = ProtoEvent {
        r#type: copy("type"),
        uid: copy("uid"),
        how: copy("how"),
        send_time: event_time(root, TimeField::Time, cache.send)?,
        start_time: event_time(root, TimeField::Start, cache.start)?,
        stale_time: event_time(root, TimeField::Stale, cache.stale)?,
        caveat: copy("caveat"),
        releaseable_to: copy("releasableTo"),
        opex: copy("opex"),
        qos: copy("qos"),
        access: copy("access"),
        ..ProtoEvent::default()
    };

    if let Some(point) = root.first_child_element("point") {
        out.lat = coordinate(point, "lat", cache.lat, 0.0);
        out.lon = coordinate(point, "lon", cache.lon, 0.0);
        out.hae = coordinate(point, "hae", cache.hae, 0.0);
        out.ce = coordinate(point, "ce", cache.ce, 999_999.0);
        out.le = coordinate(point, "le", cache.le, 999_999.0);
    }

    if let Some(detail) = root.first_child_element("detail") {
        out.detail = Some(detail_to_proto(
            detail,
            cache,
            doc.limits().max_xml_detail_bytes,
        )?);
    }

    Ok(TakMessage {
        tak_control: None,
        cot_event: Some(out),
        submission_time: positive_ms(opts.submission_ms.or(ev.submission_ms())),
        creation_time: positive_ms(opts.creation_ms.or(ev.creation_ms())),
    })
}

/// One of the three event times in milliseconds (as the unsigned proto
/// field holds a Java `long`).
fn event_time(
    root: ElementRef<'_>,
    field: TimeField,
    cached: Option<i64>,
) -> Result<u64, ConvertError> {
    if let Some(ms) = NumericCache::time(cached) {
        return Ok(ms.cast_unsigned());
    }
    match root.attribute(field.as_str()) {
        None => Ok(0),
        Some(text) => parse_cot_time(text)
            .map(i64::cast_unsigned)
            .ok_or(ConvertError::Time(field)),
    }
}

/// A `point` attribute: the remembered value, else the parsed attribute
/// (`unreadable` when it does not parse), else 0 when it is missing.
fn coordinate(point: ElementRef<'_>, name: &str, cached: Option<f64>, unreadable: f64) -> f64 {
    NumericCache::double(cached).unwrap_or_else(|| {
        point
            .attribute(name)
            .map_or(0.0, |text| parse_double(text).unwrap_or(unreadable))
    })
}

/// A creation or submission time as written to the message: only when
/// positive.
fn positive_ms(ms: Option<i64>) -> u64 {
    ms.filter(|&v| v > 0).map_or(0, i64::cast_unsigned)
}

fn detail_to_proto(
    detail: ElementRef<'_>,
    cache: &NumericCache,
    max_xml_detail_bytes: usize,
) -> Result<Detail, ConvertError> {
    let mut out = Detail::default();
    let mut converted: Vec<NodeId> = Vec::with_capacity(Typed::ALL.len());
    for kind in Typed::ALL {
        // Only the first element of each name is a candidate (K-09).
        if let Some(el) = detail.first_child_element(kind.element_name())
            && kind.extract(el, cache, &mut out)
        {
            converted.push(el.id());
        }
    }

    // Remaining child elements, each in fragment form, in document order;
    // text, comments and processing instructions directly under `detail`
    // are not carried (K-10). A fragment re-declares the namespaces it
    // uses, so the output can outgrow the input: it is capped.
    let mut xml = Vec::new();
    for child in detail.child_elements() {
        if converted.contains(&child.id()) {
            continue;
        }
        let budget = max_xml_detail_bytes.saturating_sub(xml.len());
        child
            .try_to_xml_fragment(&mut xml, budget)
            .map_err(|_| ConvertError::XmlDetailTooLarge)?;
    }
    if !xml.is_empty() {
        // The serializer writes UTF-8 only, so this never replaces anything.
        out.xml_detail = String::from_utf8_lossy(&xml).into_owned();
    }
    Ok(out)
}

/// Converts a TAK Protocol message to the `CoT` event the reference server
/// builds from it, ready for XML clients and for conversion back to proto.
///
/// - `event` attributes `version="2.0"`, `uid`, `type`, `how`, `time`,
///   `start`, `stale` always, in that order, then `caveat`, `releasableTo`,
///   `opex`, `qos`, `access` when non-empty (K-13); times in whole seconds
///   of the signed value (K-14, K-19);
/// - `point` with `lat`, `lon`, `hae`, `ce`, `le` in `Double.toString` form
///   (K-15);
/// - a `detail` element always (K-17), holding the typed messages present in
///   fixed order (K-18; `battery` as a signed `int`, K-19), then each element
///   of `xmlDetail`, which replaces the first existing element of the same
///   typed name (K-20);
/// - the numeric cache and positive creation and submission times (K-16,
///   K-22). A message without `cotEvent` is converted from the default event
///   (C-12).
///
/// `xmlDetail` is refused when longer than `limits.max_xml_detail_bytes`
/// (an exact cap), then parsed with the other `limits`; its elements count
/// against the limits of the event being built.
///
/// # Errors
///
/// [`ConvertError::XmlDetail`] for an oversized or malformed `xmlDetail`
/// (K-21); [`ConvertError::Build`] when a string holds a character XML 1.0
/// cannot carry, or the event exceeds `limits`.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SI-10 (Information Input Validation), SI-10(3) (Predictable
//   Behavior), SC-5 (Denial-of-service Protection)
// - ASD STIG: APSC-DV-002530 / V-222606, APSC-DV-002550 / V-222608,
//   APSC-DV-002590 / V-222612 (unverified)
// PARITY: src/takserver-plugins/src/main/java/tak/server/proto/StreamingProtoBufHelper.java#proto2cot
pub fn from_tak_message(msg: &TakMessage, limits: &Limits) -> Result<CotEvent, ConvertError> {
    let default_event;
    let ev = if let Some(ev) = &msg.cot_event {
        ev
    } else {
        default_event = ProtoEvent::default();
        &default_event
    };

    let mut b = Builder {
        doc: Document::with_root("event", limits).map_err(ConvertError::Build)?,
    };
    let root = b.doc.root().id();
    b.attr(root, "version", "2.0")?;
    b.attr(root, "uid", &ev.uid)?;
    b.attr(root, "type", &ev.r#type)?;
    b.attr(root, "how", &ev.how)?;
    b.attr(
        root,
        "time",
        &cot_time_to_string(ev.send_time.cast_signed()),
    )?;
    b.attr(
        root,
        "start",
        &cot_time_to_string(ev.start_time.cast_signed()),
    )?;
    b.attr(
        root,
        "stale",
        &cot_time_to_string(ev.stale_time.cast_signed()),
    )?;
    b.attr_if_set(root, "caveat", &ev.caveat)?;
    b.attr_if_set(root, "releasableTo", &ev.releaseable_to)?;
    b.attr_if_set(root, "opex", &ev.opex)?;
    b.attr_if_set(root, "qos", &ev.qos)?;
    b.attr_if_set(root, "access", &ev.access)?;

    let point = b.element(root, "point")?;
    b.attr(point, "lat", &double_to_string(ev.lat))?;
    b.attr(point, "lon", &double_to_string(ev.lon))?;
    b.attr(point, "hae", &double_to_string(ev.hae))?;
    b.attr(point, "ce", &double_to_string(ev.ce))?;
    b.attr(point, "le", &double_to_string(ev.le))?;

    let default_detail;
    let detail = if let Some(detail) = &ev.detail {
        detail
    } else {
        default_detail = Detail::default();
        &default_detail
    };
    let detail_id = b.element(root, "detail")?;
    let typed = b.typed_elements(detail_id, detail)?;
    if !detail.xml_detail.is_empty() {
        merge_xml_detail(&mut b.doc, detail_id, &detail.xml_detail, limits)?;
    }

    let cache = NumericCache {
        send: Some(ev.send_time.cast_signed()),
        start: Some(ev.start_time.cast_signed()),
        stale: Some(ev.stale_time.cast_signed()),
        lat: Some(ev.lat),
        lon: Some(ev.lon),
        hae: Some(ev.hae),
        ce: Some(ev.ce),
        le: Some(ev.le),
        battery: typed.battery,
        speed: typed.speed,
        course: typed.course,
    };
    Ok(CotEvent::from_parts(
        b.doc,
        cache,
        positive_signed(msg.creation_time),
        positive_signed(msg.submission_time),
    ))
}

/// A proto `uint64` time as the reference reads it (a Java `long`), kept
/// only when positive (K-22).
fn positive_signed(ms: u64) -> Option<i64> {
    Some(ms.cast_signed()).filter(|&v| v > 0)
}

/// Battery, speed and course as the proto gave them; `None` when the
/// message had no `Status` / `Track`.
#[derive(Default)]
struct TypedCache {
    battery: Option<i32>,
    speed: Option<f64>,
    course: Option<f64>,
}

/// Document edits for the proto-to-XML direction, mapping edit errors to
/// [`ConvertError::Build`].
struct Builder {
    doc: Document,
}

impl Builder {
    fn element(&mut self, parent: NodeId, name: &str) -> Result<NodeId, ConvertError> {
        self.doc
            .append_element(parent, name)
            .map_err(ConvertError::Build)
    }

    fn attr(&mut self, el: NodeId, name: &str, value: &str) -> Result<(), ConvertError> {
        self.doc
            .push_attribute(el, name, value)
            .map_err(ConvertError::Build)
    }

    fn attr_if_set(&mut self, el: NodeId, name: &str, value: &str) -> Result<(), ConvertError> {
        if value.is_empty() {
            Ok(())
        } else {
            self.attr(el, name, value)
        }
    }

    /// Writes the typed messages of `detail` under `detail_id` in fixed
    /// order (K-18) and returns the values the numeric cache keeps from
    /// them (K-16).
    fn typed_elements(
        &mut self,
        detail_id: NodeId,
        detail: &Detail,
    ) -> Result<TypedCache, ConvertError> {
        let mut cached = TypedCache::default();
        for kind in Typed::ALL {
            match kind {
                Typed::Contact => {
                    if let Some(c) = &detail.contact {
                        let el = self.element(detail_id, kind.element_name())?;
                        self.attr(el, "callsign", &c.callsign)?;
                        self.attr_if_set(el, "endpoint", &c.endpoint)?;
                    }
                }
                Typed::Group => {
                    if let Some(g) = &detail.group {
                        let el = self.element(detail_id, kind.element_name())?;
                        self.attr(el, "name", &g.name)?;
                        self.attr(el, "role", &g.role)?;
                    }
                }
                Typed::PrecisionLocation => {
                    if let Some(p) = &detail.precision_location {
                        let el = self.element(detail_id, kind.element_name())?;
                        self.attr(el, "geopointsrc", &p.geopointsrc)?;
                        self.attr(el, "altsrc", &p.altsrc)?;
                    }
                }
                Typed::Status => {
                    if let Some(s) = &detail.status {
                        let value = s.battery.cast_signed();
                        let el = self.element(detail_id, kind.element_name())?;
                        self.attr(el, "battery", &value.to_string())?;
                        cached.battery = Some(value);
                    }
                }
                Typed::Takv => {
                    if let Some(t) = &detail.takv {
                        let el = self.element(detail_id, kind.element_name())?;
                        self.attr(el, "device", &t.device)?;
                        self.attr(el, "platform", &t.platform)?;
                        self.attr(el, "os", &t.os)?;
                        self.attr(el, "version", &t.version)?;
                    }
                }
                Typed::Track => {
                    if let Some(t) = &detail.track {
                        let el = self.element(detail_id, kind.element_name())?;
                        self.attr(el, "speed", &double_to_string(t.speed))?;
                        self.attr(el, "course", &double_to_string(t.course))?;
                        cached.speed = Some(t.speed);
                        cached.course = Some(t.course);
                    }
                }
            }
        }
        Ok(cached)
    }
}

/// Appends the elements of a received `xmlDetail` to the event's `detail`
/// element (K-20, K-21).
///
/// The string is wrapped as the content of a `detail` document and parsed
/// with the event's limits; each child element of that wrapper (text and
/// comments between them are not carried) is copied to the end of `detail`.
/// Before a copy whose local name is one of the six typed names, the first
/// existing `detail` child with that name, typed or copied, is removed.
///
/// The accepted length is exactly `limits.max_xml_detail_bytes`: the wrapper
/// is parsed with `max_input_bytes` set to that cap plus the wrapper's own
/// length, so the wrapper never uses up the budget and a longer string is
/// refused with `xml.limit.xml_detail_bytes` before it is parsed.
///
/// Linear in the number of copied elements: the first `detail` child of each
/// typed name is tracked in a queue per name (filled in one pass, popped on
/// removal, pushed on append) instead of being searched for per copy.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SC-5 (Denial-of-service Protection), SI-10 (Information
//   Input Validation)
// - ASD STIG: APSC-DV-002530 / V-222606, APSC-DV-002590 / V-222612
//   (unverified)
// PARITY: src/takserver-plugins/src/main/java/tak/server/proto/StreamingProtoBufHelper.java#proto2cot
fn merge_xml_detail(
    doc: &mut Document,
    detail: NodeId,
    xml_detail: &str,
    limits: &Limits,
) -> Result<(), ConvertError> {
    const OPEN: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><detail>";
    const CLOSE: &str = "</detail>";
    let too_large = ConvertError::XmlDetail(XmlError::edit(XmlErrorKind::LimitExceeded(
        LimitKind::XmlDetailBytes,
    )));
    if xml_detail.len() > limits.max_xml_detail_bytes {
        return Err(too_large);
    }
    let capacity = OPEN
        .len()
        .checked_add(xml_detail.len())
        .and_then(|n| n.checked_add(CLOSE.len()))
        .ok_or(too_large)?;
    let wrapper_limits = Limits {
        max_input_bytes: limits
            .max_xml_detail_bytes
            .saturating_add(OPEN.len())
            .saturating_add(CLOSE.len()),
        ..*limits
    };
    let mut wrapped = String::with_capacity(capacity);
    wrapped.push_str(OPEN);
    wrapped.push_str(xml_detail);
    wrapped.push_str(CLOSE);
    let src = Document::parse(&wrapped, &wrapper_limits).map_err(ConvertError::XmlDetail);
    // The text is not needed once parsed; free it before the copies grow the
    // event (SEC-002).
    drop(wrapped);
    let src = src?;

    let mut first_by_name = TypedQueues::default();
    if let Some(d) = doc.element(detail) {
        for child in d.child_elements() {
            first_by_name.push(child.local_name(), child.id());
        }
    }
    for child in src.root().child_elements() {
        let name = child.local_name();
        if let Some(id) = first_by_name.pop(name) {
            doc.detach(id).map_err(ConvertError::XmlDetail)?;
        }
        let copy = doc
            .append_copy(detail, &src, child.id())
            .map_err(ConvertError::XmlDetail)?;
        first_by_name.push(name, copy);
    }
    drop(src);
    Ok(())
}

/// For each typed name, the `detail` children with that local name in
/// document order. `merge_xml_detail` only removes the first of a name and
/// only appends at the end, so a queue per name keeps the order exact.
#[derive(Default)]
struct TypedQueues([VecDeque<NodeId>; Typed::ALL.len()]);

impl TypedQueues {
    fn slot(&mut self, local_name: &str) -> Option<&mut VecDeque<NodeId>> {
        let index = Typed::ALL
            .iter()
            .position(|t| t.element_name() == local_name)?;
        self.0.get_mut(index)
    }

    /// Records `id` as the last child named `local_name`; untyped names are
    /// not tracked.
    fn push(&mut self, local_name: &str, id: NodeId) {
        if let Some(queue) = self.slot(local_name) {
            queue.push_back(id);
        }
    }

    /// Takes the first child named `local_name`, if `local_name` is typed
    /// and such a child exists.
    fn pop(&mut self, local_name: &str) -> Option<NodeId> {
        self.slot(local_name).and_then(VecDeque::pop_front)
    }
}

#[cfg(test)]
mod tests {
    use tak_proto::v1::{CotEvent as ProtoEvent, Detail, TakMessage};

    use super::from_tak_message;
    use crate::limits::Limits;
    use crate::xml::child_steps;

    fn with_xml_detail(xml_detail: String) -> TakMessage {
        TakMessage {
            cot_event: Some(ProtoEvent {
                uid: "u".into(),
                detail: Some(Detail {
                    xml_detail,
                    ..Detail::default()
                }),
                ..ProtoEvent::default()
            }),
            ..TakMessage::default()
        }
    }

    /// Child-list steps taken by one P2X conversion of `n` untyped elements
    /// followed by `n` typed ones (the QA-001 shape), and the converted
    /// `detail` children's local names.
    fn merge_steps(n: usize) -> (u64, Vec<String>) {
        let msg = with_xml_detail(format!("{}{}", "<a/>".repeat(n), "<contact/>".repeat(n)));
        let _ = child_steps::take();
        let ev = from_tak_message(&msg, &Limits::default()).unwrap();
        let steps = child_steps::take();
        let detail = ev.document().root().first_child_element("detail").unwrap();
        let names = detail
            .child_elements()
            .map(|e| e.local_name().to_owned())
            .collect();
        (steps, names)
    }

    #[test]
    fn xml_detail_merge_is_linear_in_the_element_count() {
        // QA-001: the merge used to scan `detail` from the start for every
        // typed copy, n * n steps for this shape. Counted, not timed.
        for n in [1_000_usize, 4_000] {
            let (steps, names) = merge_steps(n);
            let n64 = u64::try_from(n).unwrap();
            assert!(
                steps <= 8 * n64,
                "{steps} child steps for n = {n}; expected O(n)"
            );
            // K-20 unchanged: each contact replaced the previous one.
            assert_eq!(names.len(), n + 1);
            let (last, rest) = names.split_last().unwrap();
            assert!(rest.iter().all(|name| name == "a"));
            assert_eq!(last, "contact");
        }
        let (small, _) = merge_steps(2_000);
        let (large, _) = merge_steps(8_000);
        assert!(
            large <= 5 * small,
            "steps grew from {small} to {large} for 4x input"
        );
    }

    #[test]
    fn xml_detail_replaces_the_first_of_each_typed_name_in_order() {
        // A typed proto element, then xmlDetail copies: every typed copy
        // removes the first remaining element of its name, typed or copied.
        let mut msg = with_xml_detail(
            r#"<contact callsign="x1"/><b/><status battery="1"/><contact callsign="x2"/><p:contact xmlns:p="urn:p" callsign="x3"/>"#
                .to_owned(),
        );
        if let Some(ev) = msg.cot_event.as_mut()
            && let Some(d) = ev.detail.as_mut()
        {
            d.contact = Some(tak_proto::v1::Contact {
                callsign: "typed".into(),
                endpoint: String::new(),
            });
        }
        let ev = from_tak_message(&msg, &Limits::default()).unwrap();
        let detail = ev.document().root().first_child_element("detail").unwrap();
        let seen: Vec<(String, Option<String>)> = detail
            .child_elements()
            .map(|e| {
                (
                    e.name().to_owned(),
                    e.attribute("callsign").map(str::to_owned),
                )
            })
            .collect();
        assert_eq!(
            seen,
            vec![
                ("b".to_owned(), None),
                ("status".to_owned(), None),
                ("p:contact".to_owned(), Some("x3".to_owned())),
            ]
        );
    }
}
