// SPDX-License-Identifier: Apache-2.0

//! TAK Protocol v1 frames, as stateless and bounded pure functions.
//!
//! - **Stream** (TCP/TLS): `0xbf` ‖ varint(payload length) ‖ payload, repeated
//!   on the connection (K-27, K-28).
//! - **Mesh** (UDP/multicast, one message per datagram): `0xbf` ‖
//!   varint(protocol version = 1) ‖ `0xbf` ‖ payload (K-29).
//!
//! The payload is an encoded `TakMessage`; decode it with
//! [`crate::decode_tak_message`]. Varints are base-128, least significant
//! group first, as in protobuf. TAK-RS limits a length or version varint to 5
//! bytes and to the `u32` range, and a stream payload to the caller's
//! `max_frame`; the reference accepts any length prefix (DEV-01c). How a
//! connection reacts to an error is decided by its caller (Phase 3).
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SI-10 (Information Input Validation), SC-5 (Denial-of-service
//   Protection)
// - ASD STIG: APSC-DV-002530 / V-222606, APSC-DV-002590 / V-222612
//   (unverified)

use std::ops::Range;

/// First byte of every TAK Protocol frame.
pub const MAGIC: u8 = 0xbf;

/// The TAK Protocol version carried by mesh datagrams.
pub const TAK_PROTOCOL_VERSION: u32 = 1;

/// Longest varint accepted for a length or version (enough for any `u32`).
pub const MAX_VARINT_BYTES: usize = 5;

/// A complete stream frame found at the start of a buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Position of the payload in the buffer passed to
    /// [`decode_stream_frame`].
    pub payload: Range<usize>,
    /// Bytes the frame occupies from the start of that buffer (header and
    /// payload); the next frame starts here.
    pub consumed: usize,
}

impl Frame {
    /// The payload bytes of this frame within `buf`, the buffer it was decoded
    /// from. `None` if `buf` is shorter than the frame.
    #[must_use]
    pub fn payload_of<'a>(&self, buf: &'a [u8]) -> Option<&'a [u8]> {
        buf.get(self.payload.clone())
    }
}

/// Appends one stream frame carrying `msg` to `out`.
///
/// # Errors
///
/// [`FrameError::TooLarge`] when `msg` is longer than `max_frame` or than
/// `u32::MAX` bytes; `out` is left unchanged.
//
// PARITY: src/takserver-core/src/main/java/com/bbn/marti/nio/protocol/connections/StreamingProtoBufProtocol.java#convertGeneratedMessageV3ToProtoBufBytes
pub fn encode_stream_frame(
    msg: &[u8],
    max_frame: usize,
    out: &mut Vec<u8>,
) -> Result<(), FrameError> {
    let too_large = FrameError::TooLarge {
        len: msg.len(),
        max: max_frame,
    };
    if msg.len() > max_frame {
        return Err(too_large);
    }
    let len = u32::try_from(msg.len()).map_err(|_| too_large)?;
    out.reserve(msg.len().saturating_add(MAX_VARINT_BYTES).saturating_add(1));
    out.push(MAGIC);
    put_varint(len, out);
    out.extend_from_slice(msg);
    Ok(())
}

/// Finds the stream frame at the start of `buf`.
///
/// Returns `Ok(None)` when `buf` holds only part of a frame (read more and
/// call again with the bytes appended). The length is checked against
/// `max_frame` as soon as its varint is complete, before the payload has
/// arrived, so a caller never buffers more than one maximal frame. A zero
/// length yields an empty payload (parity-map D-07).
///
/// # Errors
///
/// [`FrameError::BadMagic`] when the first byte is not [`MAGIC`];
/// [`FrameError::VarintTooLong`] or [`FrameError::VarintOverflow`] for a bad
/// length prefix; [`FrameError::TooLarge`] when the length exceeds
/// `max_frame`.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SI-10 (Information Input Validation), SC-5 (Denial-of-service
//   Protection)
// - ASD STIG: APSC-DV-002530 / V-222606, APSC-DV-002590 / V-222612
//   (unverified)
// PARITY: src/takserver-core/src/main/java/com/bbn/marti/nio/protocol/connections/StreamingProtoBufProtocol.java#onDataReceived
pub fn decode_stream_frame(buf: &[u8], max_frame: usize) -> Result<Option<Frame>, FrameError> {
    let Some((&first, rest)) = buf.split_first() else {
        return Ok(None);
    };
    if first != MAGIC {
        return Err(FrameError::BadMagic);
    }
    let Some((len, varint_len)) = read_varint(rest)? else {
        return Ok(None);
    };
    // A u32 always fits in usize on the 32- and 64-bit targets TAK-RS builds
    // for; treat the impossible case as oversized rather than panic.
    let len = usize::try_from(len).map_err(|_| FrameError::TooLarge {
        len: usize::MAX,
        max: max_frame,
    })?;
    if len > max_frame {
        return Err(FrameError::TooLarge {
            len,
            max: max_frame,
        });
    }
    // 1 + varint_len <= 6, and len <= max_frame; checked anyway.
    let start = varint_len.checked_add(1).ok_or(FrameError::TooLarge {
        len,
        max: max_frame,
    })?;
    let end = start.checked_add(len).ok_or(FrameError::TooLarge {
        len,
        max: max_frame,
    })?;
    if buf.len() < end {
        return Ok(None);
    }
    Ok(Some(Frame {
        payload: start..end,
        consumed: end,
    }))
}

/// Appends a mesh datagram carrying `msg` to `out`.
pub fn encode_mesh(msg: &[u8], out: &mut Vec<u8>) {
    out.reserve(msg.len().saturating_add(3));
    out.push(MAGIC);
    put_varint(TAK_PROTOCOL_VERSION, out);
    out.push(MAGIC);
    out.extend_from_slice(msg);
}

/// Returns the payload of a complete mesh datagram.
///
/// The version varint must read as [`TAK_PROTOCOL_VERSION`] (a non-minimal
/// encoding of 1 is accepted, as the reference does).
///
/// # Errors
///
/// [`FrameError::BadMagic`] when either magic byte is wrong;
/// [`FrameError::Truncated`] when the datagram ends inside the header;
/// [`FrameError::VarintTooLong`] or [`FrameError::VarintOverflow`] for a bad
/// version varint; [`FrameError::UnsupportedVersion`] for any version but 1.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SI-10 (Information Input Validation)
// - ASD STIG: APSC-DV-002530 / V-222606 (unverified)
// PARITY: src/takserver-core/src/main/java/com/bbn/marti/nio/protocol/connections/SingleProtobufOrCotProtocol.java#byteBufToCot
pub fn decode_mesh(datagram: &[u8]) -> Result<&[u8], FrameError> {
    let (&first, rest) = datagram.split_first().ok_or(FrameError::Truncated)?;
    if first != MAGIC {
        return Err(FrameError::BadMagic);
    }
    let (version, varint_len) = read_varint(rest)?.ok_or(FrameError::Truncated)?;
    if version != u64::from(TAK_PROTOCOL_VERSION) {
        return Err(FrameError::UnsupportedVersion);
    }
    let rest = rest.get(varint_len..).ok_or(FrameError::Truncated)?;
    let (&second, payload) = rest.split_first().ok_or(FrameError::Truncated)?;
    if second != MAGIC {
        return Err(FrameError::BadMagic);
    }
    Ok(payload)
}

/// Reads a varint of at most [`MAX_VARINT_BYTES`] bytes and at most
/// `u32::MAX` from the start of `buf`. Returns the value and the number of
/// bytes it used, or `None` if `buf` ends before the varint does.
fn read_varint(buf: &[u8]) -> Result<Option<(u64, usize)>, FrameError> {
    let mut value: u64 = 0;
    let mut shift: u32 = 0;
    for (index, &byte) in buf.iter().take(MAX_VARINT_BYTES).enumerate() {
        // shift <= 28 here, so the 7 payload bits always fit in a u64.
        let bits = u64::from(byte & 0x7f)
            .checked_shl(shift)
            .ok_or(FrameError::VarintOverflow)?;
        value |= bits;
        if byte & 0x80 == 0 {
            if value > u64::from(u32::MAX) {
                return Err(FrameError::VarintOverflow);
            }
            // index < MAX_VARINT_BYTES, so this cannot overflow.
            return Ok(Some((value, index.saturating_add(1))));
        }
        shift = shift.saturating_add(7);
    }
    if buf.len() >= MAX_VARINT_BYTES {
        Err(FrameError::VarintTooLong)
    } else {
        Ok(None)
    }
}

/// Appends `value` as a base-128 varint.
fn put_varint(mut value: u32, out: &mut Vec<u8>) {
    loop {
        let [low, ..] = value.to_le_bytes();
        value = value.checked_shr(7).unwrap_or(0);
        if value == 0 {
            out.push(low & 0x7f);
            return;
        }
        out.push(low | 0x80);
    }
}

/// Why a frame could not be decoded or encoded.
///
/// `Display` never includes frame bytes (SI-11).
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SI-11 (Error Handling)
// - ASD STIG: APSC-DV-002570 / V-222610 (unverified)
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum FrameError {
    /// A frame does not start with (or a mesh header does not repeat)
    /// [`MAGIC`].
    #[error("TAK protocol frame does not start with the magic byte")]
    BadMagic,
    /// A varint did not end within [`MAX_VARINT_BYTES`] bytes.
    #[error("TAK protocol varint is longer than {MAX_VARINT_BYTES} bytes")]
    VarintTooLong,
    /// A varint's value is above `u32::MAX`.
    #[error("TAK protocol varint exceeds the u32 range")]
    VarintOverflow,
    /// A payload is longer than the limit.
    #[error("TAK protocol frame of {len} bytes exceeds the limit of {max} bytes")]
    TooLarge {
        /// Payload length in bytes, as declared or given.
        len: usize,
        /// The limit that was exceeded.
        max: usize,
    },
    /// A mesh datagram ends inside its header.
    #[error("TAK protocol mesh datagram is truncated")]
    Truncated,
    /// A mesh datagram carries a protocol version other than 1.
    #[error("unsupported TAK protocol version in mesh datagram")]
    UnsupportedVersion,
}

impl FrameError {
    /// The error category, for metrics and audit records.
    #[must_use]
    pub fn kind(&self) -> FrameErrorKind {
        match self {
            Self::BadMagic => FrameErrorKind::BadMagic,
            Self::VarintTooLong => FrameErrorKind::VarintTooLong,
            Self::VarintOverflow => FrameErrorKind::VarintOverflow,
            Self::TooLarge { .. } => FrameErrorKind::TooLarge,
            Self::Truncated => FrameErrorKind::Truncated,
            Self::UnsupportedVersion => FrameErrorKind::UnsupportedVersion,
        }
    }
}

/// Category of a [`FrameError`], with a stable code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FrameErrorKind {
    /// See [`FrameError::BadMagic`].
    BadMagic,
    /// See [`FrameError::VarintTooLong`].
    VarintTooLong,
    /// See [`FrameError::VarintOverflow`].
    VarintOverflow,
    /// See [`FrameError::TooLarge`].
    TooLarge,
    /// See [`FrameError::Truncated`].
    Truncated,
    /// See [`FrameError::UnsupportedVersion`].
    UnsupportedVersion,
}

impl FrameErrorKind {
    /// Stable code for metrics and audit records (e.g. `"frame.bad_magic"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BadMagic => "frame.bad_magic",
            Self::VarintTooLong => "frame.varint_too_long",
            Self::VarintOverflow => "frame.varint_overflow",
            Self::TooLarge => "frame.too_large",
            Self::Truncated => "frame.truncated",
            Self::UnsupportedVersion => "frame.unsupported_version",
        }
    }
}
