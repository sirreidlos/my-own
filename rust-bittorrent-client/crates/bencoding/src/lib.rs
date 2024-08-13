pub mod bencode;
pub mod decode;
pub mod encode;

pub use bencode::BencodeType;
pub use decode::{DecodeError, Decoder};
pub use encode::Encodable;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn encode_decode_round_trip() {
        let original = BencodeType::Dictionary({
            let mut map = BTreeMap::new();
            map.insert(b"key".to_vec(), BencodeType::Integer(123));
            map
        });

        let encoded = original.clone().encode();
        let mut decoder = Decoder::new(&encoded);
        let decoded = decoder.decode().unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn decode_encode_round_trip() {
        let original = b"d3:keyi123ee".to_vec();
        let mut decoder = Decoder::new(&original);
        let decoded = decoder.decode().unwrap();
        let re_encoded = decoded.encode();

        assert_eq!(original, re_encoded);
    }
}
