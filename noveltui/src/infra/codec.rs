use chardetng::EncodingDetector;
use std::borrow::Cow;

/// Decodes bytes to String using auto-detection (defaults to 'cn' hint).
pub fn decode_bytes(bytes: &[u8]) -> String {
    let mut det = EncodingDetector::new();
    det.feed(bytes, true);
    let encoding = det.guess(Some(b"cn"), true);
    let (cow, _, _) = encoding.decode(bytes);
    match cow {
        Cow::Borrowed(s) => s.to_string(),
        Cow::Owned(s) => s,
    }
}
