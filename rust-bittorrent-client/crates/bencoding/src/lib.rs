mod bencode;
mod decode;
mod encode;

pub use bencode::BencodeType;
pub use decode::{decode, DecodeError};
pub use encode::encode;

#[cfg(test)]
mod tests {
    use bencode::BencodeType;

    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn encode_decode_round_trip() {
        let mut map = BTreeMap::new();
        map.insert(b"key".to_vec(), BencodeType::Integer(123));

        let original = BencodeType::Dictionary(map.clone());

        let encoded = encode(map);
        let decoded = decode(encoded).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn decode_encode_round_trip() {
        let original = b"d3:keyi123ee".to_vec();
        let decoded = decode(&original).unwrap();
        let re_encoded = encode(decoded);

        assert_eq!(original, re_encoded);
    }
}
