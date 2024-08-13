use crate::bencode::BencodeType;
use std::collections::BTreeMap;

pub trait Encodable {
    fn encode(&self) -> Vec<u8>;
}

impl Encodable for Vec<u8> {
    fn encode(&self) -> Vec<u8> {
        let mut res = Vec::new();
        res.extend(format!("{}:", self.len()).into_bytes());
        res.extend(self);

        res
    }
}

impl Encodable for i64 {
    fn encode(&self) -> Vec<u8> {
        format!("i{self}e").into_bytes()
    }
}

impl Encodable for Vec<BencodeType> {
    fn encode(&self) -> Vec<u8> {
        let mut res = Vec::new();

        res.push(b'l');

        for v in self {
            res.extend(v.encode());
        }

        res.push(b'e');

        res
    }
}

impl Encodable for BTreeMap<Vec<u8>, BencodeType> {
    fn encode(&self) -> Vec<u8> {
        let mut res = Vec::new();

        res.push(b'd');

        for (k, v) in self {
            res.extend(k.encode());
            res.extend(v.encode());
        }

        res.push(b'e');

        res
    }
}

impl Encodable for BencodeType {
    fn encode(&self) -> Vec<u8> {
        match self {
            Self::ByteString(s) => s.encode(),
            Self::Integer(i) => i.encode(),
            Self::List(v) => v.encode(),
            Self::Dictionary(d) => d.encode(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn bytestring_encode() {
        let input = BencodeType::ByteString(b"spam".to_vec());
        let result = input.encode();
        assert_eq!(result, b"4:spam".to_vec());
    }

    #[test]
    fn bytestring_encode_empty() {
        let input = BencodeType::ByteString(b"".to_vec());
        let result = input.encode();
        assert_eq!(result, b"0:".to_vec());
    }

    #[test]
    fn integer_positive() {
        let input = BencodeType::Integer(3);
        let result = input.encode();
        assert_eq!(result, b"i3e".to_vec());
    }

    #[test]
    fn integer_negative() {
        let input = BencodeType::Integer(-3);
        let result = input.encode();
        assert_eq!(result, b"i-3e".to_vec());
    }

    #[test]
    fn integer_zero() {
        let input = BencodeType::Integer(0);
        let result = input.encode();
        assert_eq!(result, b"i0e".to_vec());
    }

    #[test]
    fn list_encode() {
        let input = BencodeType::List(vec![
            BencodeType::ByteString(b"spam".to_vec()),
            BencodeType::ByteString(b"eggs".to_vec()),
        ]);
        let result = input.encode();
        assert_eq!(result, b"l4:spam4:eggse".to_vec());
    }

    #[test]
    fn list_encode_empty() {
        let input = BencodeType::List(vec![]);
        let result = input.encode();
        assert_eq!(result, b"le".to_vec());
    }

    #[test]
    fn encode_dictionary_simple() {
        let mut dict = BTreeMap::new();
        dict.insert(b"cow".to_vec(), BencodeType::ByteString(b"moo".to_vec()));
        dict.insert(b"spam".to_vec(), BencodeType::ByteString(b"eggs".to_vec()));
        let input = BencodeType::Dictionary(dict);
        let result = input.encode();
        assert_eq!(result, b"d3:cow3:moo4:spam4:eggse".to_vec());
    }

    #[test]
    fn encode_dictionary_with_list() {
        let mut dict = BTreeMap::new();
        dict.insert(
            b"spam".to_vec(),
            BencodeType::List(vec![
                BencodeType::ByteString(b"a".to_vec()),
                BencodeType::ByteString(b"b".to_vec()),
            ]),
        );
        let input = BencodeType::Dictionary(dict);
        let result = input.encode();
        assert_eq!(result, b"d4:spaml1:a1:bee".to_vec());
    }

    #[test]
    fn encode_dictionary_complex() {
        let mut dict = BTreeMap::new();
        dict.insert(
            b"publisher".to_vec(),
            BencodeType::ByteString(b"bob".to_vec()),
        );
        dict.insert(
            b"publisher-webpage".to_vec(),
            BencodeType::ByteString(b"www.example.com".to_vec()),
        );
        dict.insert(
            b"publisher.location".to_vec(),
            BencodeType::ByteString(b"home".to_vec()),
        );
        let input = BencodeType::Dictionary(dict);
        let result = input.encode();
        assert_eq!(
            result,
            b"d9:publisher3:bob17:publisher-webpage15:www.example.com18:publisher.location4:homee"
                .to_vec()
        );
    }

    #[test]
    fn encode_empty_dictionary() {
        let input = BencodeType::Dictionary(BTreeMap::new());
        let result = input.encode();
        assert_eq!(result, b"de".to_vec());
    }
}
