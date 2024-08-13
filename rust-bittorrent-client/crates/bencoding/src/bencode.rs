use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BencodeType {
    ByteString(Vec<u8>),
    Integer(i64),
    List(Vec<BencodeType>),
    Dictionary(BTreeMap<Vec<u8>, BencodeType>),
}
