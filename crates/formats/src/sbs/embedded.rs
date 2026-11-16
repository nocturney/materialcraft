use crate::{FormatError, Result};
use base64::Engine as _;
use std::io::{Read, Write};

pub(super) fn base64_engine() -> base64::engine::GeneralPurpose {
    base64::engine::general_purpose::STANDARD
}

pub(super) fn decode_embedded(strdata: &str) -> std::result::Result<Vec<u8>, String> {
    let raw = base64_engine().decode(strdata.split_whitespace().collect::<String>()).map_err(|e| format!("bad base64: {e}"))?;
    let (length, compressed) = raw.split_at_checked(4).ok_or("too short")?;
    let length = u32::from_be_bytes([length[0], length[1], length[2], length[3]]) as usize;
    if length as u64 > crate::MAX_RESOURCE {
        return Err("embedded resource exceeds the 256 MiB limit".into());
    }

    let mut out = Vec::new();

    // Reading one extra byte detects data that contradicts the declared format length
    let mut decoder = flate2::read::ZlibDecoder::new(compressed).take(length as u64 + 1);
    decoder.read_to_end(&mut out).map_err(|e| format!("bad compressed data: {e}"))?;
    if out.len() != length {
        return Err(format!("{} bytes where {length} were declared", out.len()));
    }

    Ok(out)
}

pub(super) fn encode_embedded(bytes: &[u8]) -> Result<String> {
    let length = u32::try_from(bytes.len()).map_err(|_| FormatError::Io("resource too large".into()))?;
    let mut encoder = flate2::write::ZlibEncoder::new(length.to_be_bytes().to_vec(), flate2::Compression::default());
    encoder.write_all(bytes).map_err(|e| FormatError::Io(e.to_string()))?;
    let raw = encoder.finish().map_err(|e| FormatError::Io(e.to_string()))?;
    Ok(craft_mime::transfer::base64(&raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_encoder_preserves_embedded_wire_bytes() {
        for len in 0..=513 {
            let bytes: Vec<u8> = (0..len).map(|i| (i % 256) as u8).collect();
            let mut encoder = flate2::write::ZlibEncoder::new((len as u32).to_be_bytes().to_vec(), flate2::Compression::default());
            encoder.write_all(&bytes).unwrap();
            let raw = encoder.finish().unwrap();
            assert_eq!(encode_embedded(&bytes).unwrap(), base64_engine().encode(&raw));
            assert_eq!(decode_embedded(&encode_embedded(&bytes).unwrap()).unwrap(), bytes);
        }
    }

    #[test]
    fn strict_decoder_policy_is_not_replaced_by_mime_tolerance() {
        assert!(decode_embedded("!!!!").unwrap_err().starts_with("bad base64:"));
        assert!(decode_embedded("AA").unwrap_err().starts_with("bad base64:"));
        let encoded = encode_embedded(b"payload").unwrap();
        let spaced = encoded.chars().map(|c| format!("{c} \n")).collect::<String>();
        assert_eq!(decode_embedded(&spaced).unwrap(), b"payload");
    }

    #[test]
    fn decompression_requires_the_declared_length() {
        let encoded = encode_embedded(&[1, 2, 3, 4]).unwrap();

        assert_eq!(decode_embedded(&encoded).unwrap(), [1, 2, 3, 4]);

        let mut raw = base64_engine().decode(&encoded).unwrap();
        for length in [1_u32, 5] {
            raw[..4].copy_from_slice(&length.to_be_bytes());

            assert!(decode_embedded(&base64_engine().encode(&raw)).unwrap_err().contains("declared"));
        }

        raw[..4].copy_from_slice(&u32::MAX.to_be_bytes());

        assert!(decode_embedded(&base64_engine().encode(&raw)).unwrap_err().contains("256 MiB limit"));
    }
}
