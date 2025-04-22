use std::u16;

use crate::utils;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DnsType {
    A = 1,
    NS,
    MD, //
    MF,
    Cname,
    Soa,
    MB,
    MG,
    MR,
    Null,
    Wks,
    Ptr,
    Hinfo,
    Minfo,
    MX,
    Txt,

    // Types that appear only in question part of a query.
    Axfr = 252,
    Mailb,
    Maila,
    AllRecords,
}

impl From<u16> for DnsType {
    fn from(value: u16) -> Self {
        match value {
            1 => DnsType::A,
            2 => DnsType::NS,
            3 => DnsType::MD,
            4 => DnsType::MF,
            5 => DnsType::Cname,
            6 => DnsType::Soa,
            7 => DnsType::MB,
            8 => DnsType::MG,
            9 => DnsType::MR,
            10 => DnsType::Null,
            11 => DnsType::Wks,
            12 => DnsType::Ptr,
            13 => DnsType::Hinfo,
            14 => DnsType::Minfo,
            15 => DnsType::MX,
            16 => DnsType::Txt,
            252 => DnsType::Axfr,
            253 => DnsType::Mailb,
            254 => DnsType::Maila,
            255 => DnsType::AllRecords,
            _ => unreachable!("Couldn't find DNS type for value: {value}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DnsClass {
    IN = 1,
    CS,
    CH,
    HS,
    // Any Class only appear in question section of a Query
    AnyClass = 255,
}

impl From<u16> for DnsClass {
    fn from(value: u16) -> Self {
        match value {
            1 => DnsClass::IN,
            2 => DnsClass::CS,
            3 => DnsClass::CH,
            4 => DnsClass::HS,
            255 => DnsClass::AnyClass,
            _ => unreachable!("Couldn't find DNS Class for value: {value}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DnsRecord {
    pub name: String,
    pub dns_type: DnsType,
    pub dns_class: DnsClass,
    pub time_to_live: i32,
    pub rd_length: u16,
    pub rd_data: Vec<u8>,
}

pub fn serialize_record(record: &DnsRecord) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();

    for label in record.name.split(".") {
        let len = label.len() as u8;
        let content = label.as_bytes();
        bytes.push(len);
        bytes.extend_from_slice(content);
    }
    bytes.push(0);

    let serialize_u16 = |bytes: &mut Vec<u8>, value: u16| {
        bytes.extend_from_slice(&[(value >> 8) as u8, value as u8])
    };

    serialize_u16(&mut bytes, record.dns_type as u16);
    serialize_u16(&mut bytes, record.dns_class as u16);
    // // TTL 32bits
    if record.time_to_live > 0 {
        bytes.extend_from_slice(&[
            (record.time_to_live >> 24) as u8,
            (record.time_to_live >> 16) as u8,
            (record.time_to_live >> 8) as u8,
            record.time_to_live as u8,
        ]);
    }

    // // RDLENGTH 16 bits
    if record.rd_length > 0 {
        serialize_u16(&mut bytes, record.rd_length);
        bytes.extend(&record.rd_data);
    }

    bytes
}

pub fn deserialize_record(bytes: &[u8], mut begin: usize) -> (DnsRecord, usize) {
    let mut end;
    let mut name;

    let size = bytes[begin - 1] as usize;
    (begin, end, name) = deserialize_label(bytes, begin, begin + size);

    while begin != end {
        let label;
        (begin, end, label) = deserialize_label(bytes, begin, end);
        name = format!("{name}.{label}");
        println!("name: {name}");
    }
    let dns_type_value = utils::double_u8_to_u16(bytes, end);
    end += 2;
    let dns_class_value = utils::double_u8_to_u16(bytes, end);
    end += 2;

    (
        DnsRecord {
            name,
            dns_type: dns_class_value.into(),
            dns_class: dns_type_value.into(),
            time_to_live: 0,
            rd_length: 0,
            rd_data: Vec::new(),
        },
        end,
    )
}

fn deserialize_label(bytes: &[u8], mut begin: usize, mut end: usize) -> (usize, usize, String) {
    let pointer_bytes = utils::double_u8_to_u16(bytes, begin - 1);
    let is_a_pointer = pointer_bytes & (0b11 << 14) != 0;
    if is_a_pointer {
        let offset = (pointer_bytes & 0b0011111111111111) as usize;
        begin = offset + 1;
        let size = bytes[begin - 1] as usize;
        end = begin + size;
    }

    let sub_string = String::from_utf8_lossy(&bytes[begin..end]).to_string();
    let begin = end + 1;
    let size = bytes[end] as usize;
    let end = begin + size;

    (begin, end, sub_string)
}

#[test]
fn test_deserialize() {
    let response = [
        12, 99, 111, 100, 101, 99, 114, 97, 102, 116, 101, 114, 115, 2, 105, 111, 0, 0, 1, 0, 1,
    ];
    let (record, _) = deserialize_record(&response, 1);

    assert_eq!("codecrafters.io".to_string(), record.name);
    assert_eq!(DnsType::A, record.dns_type);
    assert_eq!(DnsClass::IN, record.dns_class);
    println!("record: {record:?}");
}

#[test]
fn test_serialize() {
    let response = [
        12, 99, 111, 100, 101, 99, 114, 97, 102, 116, 101, 114, 115, 2, 105, 111, 0, 0, 1, 0, 1,
    ]
    .to_vec();

    let record = DnsRecord {
        name: "codecrafters.io".to_string(),
        dns_type: DnsType::A,
        dns_class: DnsClass::IN,
        rd_data: Vec::new(),
        rd_length: 0,
        time_to_live: 0,
    };

    let result = serialize_record(&record);

    assert_eq!(result, response);
}
