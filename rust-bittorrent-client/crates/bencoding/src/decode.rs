use std::{collections::BTreeMap, str::Utf8Error};

use crate::bencode::BencodeType;

#[derive(Debug)]
pub struct Decoder<'a> {
    input: &'a [u8],
    cursor: usize,
}

#[derive(Debug)]
pub enum DecodeError {
    InvalidUtf8(Utf8Error),
    InvalidInteger,
    UnexpectedEndOfInput,
    UnexpectedCharacter(u8),
    UnexpectedFormat,
}

impl From<Utf8Error> for DecodeError {
    fn from(value: Utf8Error) -> Self {
        Self::InvalidUtf8(value)
    }
}

impl<'a> Decoder<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, cursor: 0 }
    }

    pub fn decode(&mut self) -> Result<BencodeType, DecodeError> {
        match self.input[self.cursor] {
            b'i' => self.decode_integer(),
            b'l' => self.decode_list(),
            b'd' => self.decode_dictionary(),
            b'0'..=b'9' => self.decode_bytestring(),
            c => Err(DecodeError::UnexpectedCharacter(c)),
        }
    }

    fn consume_byte(&mut self) {
        self.cursor += 1;
    }

    fn decode_bytestring(&mut self) -> Result<BencodeType, DecodeError> {
        let start = self.cursor;
        while self.input.get(self.cursor) != Some(&b':') {
            self.cursor += 1;

            if self.cursor >= self.input.len() {
                return Err(DecodeError::UnexpectedEndOfInput);
            }
        }

        // Cursor now points to the colon
        let string_len: usize = std::str::from_utf8(&self.input[start..self.cursor])?
            .parse()
            .map_err(|_| DecodeError::InvalidInteger)?;

        self.consume_byte(); // Consume colon byte

        let string_start = self.cursor;
        self.cursor += string_len;

        Ok(BencodeType::ByteString(
            self.input[string_start..self.cursor].to_vec(),
        ))
    }

    fn decode_integer(&mut self) -> Result<BencodeType, DecodeError> {
        self.consume_byte(); // skip 'i'
        let start = self.cursor;

        while self.input.get(self.cursor) != Some(&b'e') {
            self.cursor += 1;

            if self.cursor >= self.input.len() {
                return Err(DecodeError::UnexpectedEndOfInput);
            }
        }

        let integer_str = std::str::from_utf8(&self.input[start..self.cursor])?;
        self.consume_byte(); // skip 'e'

        // leading zeros
        if integer_str.len() > 1 && integer_str.starts_with('0') {
            return Err(DecodeError::InvalidInteger);
        }

        // negative zero
        if integer_str == "-0" {
            return Err(DecodeError::InvalidInteger);
        }

        let integer = integer_str
            .parse()
            .map_err(|_| DecodeError::InvalidInteger)?;

        Ok(BencodeType::Integer(integer))
    }

    fn decode_list(&mut self) -> Result<BencodeType, DecodeError> {
        self.consume_byte(); // skip 'l'
        let mut res = Vec::new();

        while self.input.get(self.cursor) != Some(&b'e') {
            res.push(self.decode()?);

            if self.cursor >= self.input.len() {
                return Err(DecodeError::UnexpectedEndOfInput);
            }
        }

        self.consume_byte(); // skip 'e'

        Ok(BencodeType::List(res))
    }

    fn decode_dictionary(&mut self) -> Result<BencodeType, DecodeError> {
        self.consume_byte(); // skip 'd'
        let mut res: BTreeMap<Vec<u8>, BencodeType> = BTreeMap::new();

        while self.input.get(self.cursor) != Some(&b'e') {
            let k = self.decode_bytestring()?;
            let k_inner = if let BencodeType::ByteString(inner) = k {
                inner
            } else {
                return Err(DecodeError::UnexpectedFormat);
            };
            let v = self.decode()?;

            res.insert(k_inner, v);

            if self.cursor >= self.input.len() {
                return Err(DecodeError::UnexpectedEndOfInput);
            }
        }

        self.consume_byte(); // skip 'e'

        Ok(BencodeType::Dictionary(res))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::bencode::BencodeType;

    use super::*;

    #[test]
    fn integer_decode_int() {
        let input = vec![b'i', b'3', b'e'];
        let mut decoder = Decoder::new(&input);
        let result = decoder.decode().unwrap();
        assert_eq!(result, BencodeType::Integer(3));
    }

    #[test]
    fn integer_decode_negative() {
        let input = vec![b'i', b'-', b'3', b'e'];
        let mut decoder = Decoder::new(&input);
        let result = decoder.decode().unwrap();
        assert_eq!(result, BencodeType::Integer(-3));
    }

    #[test]
    #[should_panic(expected = "InvalidInteger")]
    // i-0e is invalid.
    fn integer_decode_negative_zero() {
        let input = vec![b'i', b'-', b'0', b'e'];
        let mut decoder = Decoder::new(&input);
        decoder.decode().unwrap();
    }

    #[test]
    #[should_panic(expected = "InvalidInteger")]
    // Encodings with a leading zero are invalid (other than i0e).
    fn integer_decode_leading_zero() {
        let input = vec![b'i', b'0', b'3', b'e'];
        let mut decoder = Decoder::new(&input);
        decoder.decode().unwrap();
    }

    #[test]
    fn integer_decode_zero() {
        let input = vec![b'i', b'0', b'e'];
        let mut decoder = Decoder::new(&input);
        let result = decoder.decode().unwrap();
        assert_eq!(result, BencodeType::Integer(0));
    }

    #[test]
    fn bytestring_decode_spam() {
        let input = vec![b'4', b':', b's', b'p', b'a', b'm'];
        let mut decoder = Decoder::new(&input);
        let result = decoder.decode().unwrap();
        assert_eq!(result, BencodeType::ByteString(b"spam".to_vec()));
    }

    #[test]
    fn bytestring_decode_empty() {
        let input = vec![b'0', b':'];
        let mut decoder = Decoder::new(&input);
        let result = decoder.decode().unwrap();
        assert_eq!(result, BencodeType::ByteString(b"".to_vec()));
    }

    #[test]
    fn list_decode() {
        let input = vec![b'l', b'i', b'3', b'e', b'i', b'4', b'e', b'e'];
        let mut decoder = Decoder::new(&input);
        let result = decoder.decode().unwrap();
        assert_eq!(
            result,
            BencodeType::List(vec![BencodeType::Integer(3), BencodeType::Integer(4)])
        );
    }

    #[test]
    fn list_decode_spam_eggs() {
        let input = vec![
            b'l', b'4', b':', b's', b'p', b'a', b'm', b'4', b':', b'e', b'g', b'g', b's', b'e',
        ];
        let mut decoder = Decoder::new(&input);
        let result = decoder.decode().unwrap();
        assert_eq!(
            result,
            BencodeType::List(vec![
                BencodeType::ByteString(b"spam".to_vec()),
                BencodeType::ByteString(b"eggs".to_vec())
            ])
        );
    }

    #[test]
    fn list_decode_empty() {
        let input = vec![b'l', b'e'];
        let mut decoder = Decoder::new(&input);
        let result = decoder.decode().unwrap();
        assert_eq!(result, BencodeType::List(vec![]));
    }

    #[test]
    fn dictionary_decode() {
        let input = vec![b'd', b'3', b':', b'f', b'o', b'o', b'i', b'3', b'e', b'e'];
        let mut decoder = Decoder::new(&input);
        let result = decoder.decode().unwrap();
        let mut expected_dict = BTreeMap::new();
        expected_dict.insert(b"foo".to_vec(), BencodeType::Integer(3));
        assert_eq!(result, BencodeType::Dictionary(expected_dict));
    }

    #[test]
    fn dictionary_decode_strings() {
        let input = vec![
            b'd', b'3', b':', b'c', b'o', b'w', b'3', b':', b'm', b'o', b'o', b'4', b':', b's',
            b'p', b'a', b'm', b'4', b':', b'e', b'g', b'g', b's', b'e',
        ];
        let mut decoder = Decoder::new(&input);
        let result = decoder.decode().unwrap();

        let mut expected_dict = BTreeMap::new();
        expected_dict.insert(b"cow".to_vec(), BencodeType::ByteString(b"moo".to_vec()));
        expected_dict.insert(b"spam".to_vec(), BencodeType::ByteString(b"eggs".to_vec()));

        assert_eq!(result, BencodeType::Dictionary(expected_dict));
    }

    #[test]
    fn dictionary_decode_list_nested() {
        let input = vec![
            b'd', b'4', b':', b's', b'p', b'a', b'm', b'l', b'1', b':', b'a', b'1', b':', b'b',
            b'e', b'e',
        ];
        let mut decoder = Decoder::new(&input);
        let result = decoder.decode().unwrap();

        let mut expected_dict = BTreeMap::new();
        expected_dict.insert(
            b"spam".to_vec(),
            BencodeType::List(vec![
                BencodeType::ByteString(b"a".to_vec()),
                BencodeType::ByteString(b"b".to_vec()),
            ]),
        );

        assert_eq!(result, BencodeType::Dictionary(expected_dict));
    }

    #[test]
    fn dictionary_decode_long() {
        let input = vec![
            b'd', b'9', b':', b'p', b'u', b'b', b'l', b'i', b's', b'h', b'e', b'r', b'3', b':',
            b'b', b'o', b'b', b'1', b'7', b':', b'p', b'u', b'b', b'l', b'i', b's', b'h', b'e',
            b'r', b'-', b'w', b'e', b'b', b'p', b'a', b'g', b'e', b'1', b'5', b':', b'w', b'w',
            b'w', b'.', b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'c', b'o', b'm', b'1',
            b'8', b':', b'p', b'u', b'b', b'l', b'i', b's', b'h', b'e', b'r', b'.', b'l', b'o',
            b'c', b'a', b't', b'i', b'o', b'n', b'4', b':', b'h', b'o', b'm', b'e', b'e',
        ];
        let mut decoder = Decoder::new(&input);
        let result = decoder.decode().unwrap();

        let mut expected_dict = BTreeMap::new();
        expected_dict.insert(
            b"publisher".to_vec(),
            BencodeType::ByteString(b"bob".to_vec()),
        );
        expected_dict.insert(
            b"publisher-webpage".to_vec(),
            BencodeType::ByteString(b"www.example.com".to_vec()),
        );
        expected_dict.insert(
            b"publisher.location".to_vec(),
            BencodeType::ByteString(b"home".to_vec()),
        );

        assert_eq!(result, BencodeType::Dictionary(expected_dict));
    }

    #[test]
    fn dictionary_decode_empty() {
        let input = vec![b'd', b'e'];
        let mut decoder = Decoder::new(&input);
        let result = decoder.decode().unwrap();

        let expected_dict = BTreeMap::new();

        assert_eq!(result, BencodeType::Dictionary(expected_dict));
    }
}
