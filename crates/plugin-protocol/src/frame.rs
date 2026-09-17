use std::io::{self, Read, Write};

use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use thiserror::Error;

use crate::{ProtocolVersion, ValidationError, canonical_json, canonical_json_value};

const FRAME_MAGIC: [u8; 4] = *b"MKPN";
pub const FIXED_FRAME_HEADER_BYTES: usize = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameKind {
    Request = 1,
    Response = 2,
    Event = 3,
    Cancel = 4,
    Shutdown = 5,
}

impl TryFrom<u8> for FrameKind {
    type Error = FrameError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Request),
            2 => Ok(Self::Response),
            3 => Ok(Self::Event),
            4 => Ok(Self::Cancel),
            5 => Ok(Self::Shutdown),
            _ => Err(FrameError::UnknownKind(value)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameLimits {
    pub max_header_bytes: u32,
    pub max_body_bytes: u64,
}

impl Default for FrameLimits {
    fn default() -> Self {
        Self {
            max_header_bytes: 64 * 1024,
            max_body_bytes: 32 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub protocol: ProtocolVersion,
    pub kind: FrameKind,
    pub request_id: u64,
    pub header: Vec<u8>,
    pub body: Vec<u8>,
}

impl Frame {
    pub fn from_header<T: Serialize>(
        protocol: ProtocolVersion,
        kind: FrameKind,
        request_id: u64,
        header: &T,
        body: Vec<u8>,
    ) -> Result<Self, FrameError> {
        Ok(Self {
            protocol,
            kind,
            request_id,
            header: canonical_json(header)?,
            body,
        })
    }

    pub fn decode_header<T: DeserializeOwned>(&self) -> Result<T, FrameError> {
        serde_json::from_slice(&self.header).map_err(FrameError::InvalidHeader)
    }
}

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("I/O failure while reading or writing a frame")]
    Io(#[from] io::Error),
    #[error("frame magic is invalid")]
    InvalidMagic,
    #[error("frame reserved flags are non-zero")]
    ReservedFlags,
    #[error("unknown frame kind {0}")]
    UnknownKind(u8),
    #[error("frame header is larger than the configured limit")]
    HeaderTooLarge,
    #[error("frame body is larger than the configured limit")]
    BodyTooLarge,
    #[error("frame length does not fit this platform")]
    LengthOverflow,
    #[error("frame header is invalid JSON")]
    InvalidHeader(#[source] serde_json::Error),
    #[error("frame header is not canonical JSON")]
    NonCanonicalHeader,
    #[error("frame header cannot be canonicalized")]
    Canonical(#[from] ValidationError),
}

pub fn write_frame<W: Write>(
    writer: &mut W,
    frame: &Frame,
    limits: FrameLimits,
) -> Result<(), FrameError> {
    let header_len = u32::try_from(frame.header.len()).map_err(|_| FrameError::LengthOverflow)?;
    let body_len = u64::try_from(frame.body.len()).map_err(|_| FrameError::LengthOverflow)?;
    if header_len > limits.max_header_bytes {
        return Err(FrameError::HeaderTooLarge);
    }
    if body_len > limits.max_body_bytes {
        return Err(FrameError::BodyTooLarge);
    }
    validate_canonical_header(&frame.header)?;

    let mut fixed = [0u8; FIXED_FRAME_HEADER_BYTES];
    fixed[0..4].copy_from_slice(&FRAME_MAGIC);
    fixed[4..6].copy_from_slice(&frame.protocol.major.to_le_bytes());
    fixed[6..8].copy_from_slice(&frame.protocol.minor.to_le_bytes());
    fixed[8] = frame.kind as u8;
    fixed[9] = 0;
    fixed[10..18].copy_from_slice(&frame.request_id.to_le_bytes());
    fixed[18..22].copy_from_slice(&header_len.to_le_bytes());
    fixed[22..30].copy_from_slice(&body_len.to_le_bytes());

    writer.write_all(&fixed)?;
    writer.write_all(&frame.header)?;
    writer.write_all(&frame.body)?;
    writer.flush()?;
    Ok(())
}

pub fn read_frame<R: Read>(
    reader: &mut R,
    limits: FrameLimits,
) -> Result<Option<Frame>, FrameError> {
    let mut fixed = [0u8; FIXED_FRAME_HEADER_BYTES];
    let first = reader.read(&mut fixed[..1])?;
    if first == 0 {
        return Ok(None);
    }
    reader.read_exact(&mut fixed[1..])?;
    if fixed[0..4] != FRAME_MAGIC {
        return Err(FrameError::InvalidMagic);
    }
    if fixed[9] != 0 {
        return Err(FrameError::ReservedFlags);
    }

    let protocol = ProtocolVersion {
        major: u16::from_le_bytes([fixed[4], fixed[5]]),
        minor: u16::from_le_bytes([fixed[6], fixed[7]]),
    };
    let kind = FrameKind::try_from(fixed[8])?;
    let request_id = u64::from_le_bytes(fixed[10..18].try_into().expect("fixed range"));
    let header_len = u32::from_le_bytes(fixed[18..22].try_into().expect("fixed range"));
    let body_len = u64::from_le_bytes(fixed[22..30].try_into().expect("fixed range"));
    if header_len > limits.max_header_bytes {
        return Err(FrameError::HeaderTooLarge);
    }
    if body_len > limits.max_body_bytes {
        return Err(FrameError::BodyTooLarge);
    }

    let mut header = vec![0; usize::try_from(header_len).map_err(|_| FrameError::LengthOverflow)?];
    let mut body = vec![0; usize::try_from(body_len).map_err(|_| FrameError::LengthOverflow)?];
    reader.read_exact(&mut header)?;
    validate_canonical_header(&header)?;
    reader.read_exact(&mut body)?;
    Ok(Some(Frame {
        protocol,
        kind,
        request_id,
        header,
        body,
    }))
}

fn validate_canonical_header(header: &[u8]) -> Result<(), FrameError> {
    let value: Value = serde_json::from_slice(header).map_err(FrameError::InvalidHeader)?;
    let canonical = canonical_json_value(&value)?;
    if canonical != header {
        return Err(FrameError::NonCanonicalHeader);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Header {
        operation: String,
        value: u32,
    }

    #[test]
    fn frame_round_trip_keeps_raw_body() {
        let expected_header = Header {
            operation: "render".to_owned(),
            value: 42,
        };
        let frame = Frame::from_header(
            ProtocolVersion::V1_0,
            FrameKind::Response,
            17,
            &expected_header,
            vec![0, 1, 2, 255],
        )
        .unwrap();
        let mut bytes = Vec::new();
        write_frame(&mut bytes, &frame, FrameLimits::default()).unwrap();
        let decoded = read_frame(&mut Cursor::new(bytes), FrameLimits::default())
            .unwrap()
            .unwrap();
        assert_eq!(decoded.kind, FrameKind::Response);
        assert_eq!(decoded.request_id, 17);
        assert_eq!(decoded.decode_header::<Header>().unwrap(), expected_header);
        assert_eq!(decoded.body, [0, 1, 2, 255]);
    }

    #[test]
    fn oversize_lengths_fail_before_allocation() {
        let mut bytes = [0u8; FIXED_FRAME_HEADER_BYTES];
        bytes[0..4].copy_from_slice(&FRAME_MAGIC);
        bytes[4..6].copy_from_slice(&1u16.to_le_bytes());
        bytes[8] = FrameKind::Request as u8;
        bytes[18..22].copy_from_slice(&(65 * 1024u32).to_le_bytes());
        let error = read_frame(&mut Cursor::new(bytes), FrameLimits::default()).unwrap_err();
        assert!(matches!(error, FrameError::HeaderTooLarge));
    }

    #[test]
    fn truncated_frame_is_an_io_error() {
        let frame = Frame::from_header(
            ProtocolVersion::V1_0,
            FrameKind::Request,
            1,
            &serde_json::json!({"operation": "ping"}),
            vec![1, 2, 3],
        )
        .unwrap();
        let mut bytes = Vec::new();
        write_frame(&mut bytes, &frame, FrameLimits::default()).unwrap();
        bytes.pop();
        assert!(matches!(
            read_frame(&mut Cursor::new(bytes), FrameLimits::default()),
            Err(FrameError::Io(_))
        ));
    }

    #[test]
    fn noncanonical_header_is_rejected() {
        let mut fixed = [0u8; FIXED_FRAME_HEADER_BYTES];
        fixed[0..4].copy_from_slice(&FRAME_MAGIC);
        fixed[4..6].copy_from_slice(&1u16.to_le_bytes());
        fixed[8] = FrameKind::Request as u8;
        let header = br#"{ "z": 1, "a": 2 }"#;
        fixed[18..22].copy_from_slice(&(header.len() as u32).to_le_bytes());
        let bytes = fixed
            .into_iter()
            .chain(header.iter().copied())
            .collect::<Vec<_>>();
        assert!(matches!(
            read_frame(&mut Cursor::new(bytes), FrameLimits::default()),
            Err(FrameError::NonCanonicalHeader)
        ));
    }

    #[test]
    fn adversarial_fixed_headers_fail_without_body_allocation() {
        let mut cases = Vec::new();

        let mut bad_magic = [0u8; FIXED_FRAME_HEADER_BYTES];
        bad_magic[0..4].copy_from_slice(b"NOPE");
        cases.push((bad_magic, "magic"));

        let mut unknown_kind = [0u8; FIXED_FRAME_HEADER_BYTES];
        unknown_kind[0..4].copy_from_slice(&FRAME_MAGIC);
        unknown_kind[8] = 255;
        cases.push((unknown_kind, "kind"));

        let mut reserved = [0u8; FIXED_FRAME_HEADER_BYTES];
        reserved[0..4].copy_from_slice(&FRAME_MAGIC);
        reserved[8] = FrameKind::Request as u8;
        reserved[9] = 1;
        cases.push((reserved, "reserved"));

        let mut huge_body = [0u8; FIXED_FRAME_HEADER_BYTES];
        huge_body[0..4].copy_from_slice(&FRAME_MAGIC);
        huge_body[8] = FrameKind::Request as u8;
        huge_body[22..30].copy_from_slice(&(u64::MAX).to_le_bytes());
        cases.push((huge_body, "body"));

        for (bytes, label) in cases {
            let result = read_frame(&mut Cursor::new(bytes), FrameLimits::default());
            assert!(result.is_err(), "{label} adversarial header was accepted");
        }
    }
}
