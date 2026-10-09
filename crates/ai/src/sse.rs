use crate::AiError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub event: String,
    pub data: String,
}

/// Incremental SSE parser: never decode a partial UTF-8 line or dispatch partial JSON.
pub struct Decoder {
    pending: Vec<u8>,
    event: String,
    data: Vec<String>,
    size: usize,
    limit: usize,
}
impl Decoder {
    pub fn new(limit: usize) -> Self {
        Self {
            pending: Vec::new(),
            event: String::new(),
            data: Vec::new(),
            size: 0,
            limit,
        }
    }
    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<Frame>, AiError> {
        let mut frames = Vec::new();
        for &byte in bytes {
            if self.pending.len() >= self.limit {
                return Err(AiError::Limit);
            }
            self.pending.push(byte);
            if byte != b'\n' {
                continue;
            }
            let mut line = std::mem::take(&mut self.pending);
            line.pop();
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            let text = std::str::from_utf8(&line).map_err(|_| AiError::Protocol)?;
            if text.is_empty() {
                if !self.data.is_empty() {
                    frames.push(Frame {
                        event: std::mem::take(&mut self.event),
                        data: self.data.join("\n"),
                    });
                }
                self.data.clear();
                self.event.clear();
                self.size = 0;
            } else if !text.starts_with(':') {
                let (field, value) = text.split_once(':').unwrap_or((text, ""));
                let value = value.strip_prefix(' ').unwrap_or(value);
                self.size = self.size.checked_add(value.len()).ok_or(AiError::Limit)?;
                if self.size > self.limit {
                    return Err(AiError::Limit);
                }
                match field {
                    "event" => self.event = value.into(),
                    "data" => self.data.push(value.into()),
                    _ => {}
                }
            }
        }
        Ok(frames)
    }
    pub fn finish(&self) -> Result<(), AiError> {
        if self.pending.is_empty() && self.data.is_empty() {
            Ok(())
        } else {
            Err(AiError::Protocol)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_byte_split_preserves_utf8_crlf_and_multiline() {
        let bytes = ":keepalive\r\nevent: text\r\ndata: 中文🙂\r\ndata: next\r\n\r\n".as_bytes();
        for split in 0..=bytes.len() {
            let mut decoder = Decoder::new(1024);
            let mut frames = decoder.push(&bytes[..split]).unwrap();
            frames.extend(decoder.push(&bytes[split..]).unwrap());
            assert_eq!(
                frames,
                vec![Frame {
                    event: "text".into(),
                    data: "中文🙂\nnext".into()
                }]
            );
            decoder.finish().unwrap();
        }
    }
    #[test]
    fn oversized_invalid_and_unterminated_frames_fail() {
        assert_eq!(
            Decoder::new(4).push(b"data: too much\n\n"),
            Err(AiError::Limit)
        );
        assert_eq!(
            Decoder::new(100).push(b"data: \xff\n\n"),
            Err(AiError::Protocol)
        );
        let mut decoder = Decoder::new(100);
        decoder.push(b"data: partial\n").unwrap();
        assert_eq!(decoder.finish(), Err(AiError::Protocol));
    }
}
