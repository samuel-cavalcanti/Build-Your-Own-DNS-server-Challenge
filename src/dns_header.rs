use crate::utils;

#[derive(Debug, PartialEq)]
pub struct DnsHeader {
    pub id: u16,
    pub query: QR,
    pub op_code: OpCode,
    pub aa: bool,
    pub tc: bool,
    pub rd: bool,
    pub ra: bool,
    pub z: u8,
    pub response_code: ResponseCode,
    pub questions_count: u16,
    pub answers_count: u16,
    pub authority_count: u16,
    pub additional_count: u16,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum QR {
    Query,
    Response,
}
impl From<u8> for QR {
    fn from(value: u8) -> Self {
        if value > 0 {
            QR::Response
        } else {
            QR::Query
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OpCode {
    StandardQuery = 0,
    InverseQuery = 1,
    ServerStatus = 2,
    NotImplemented,
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::StandardQuery,
            1 => Self::InverseQuery,
            2 => Self::ServerStatus,
            _ => Self::NotImplemented,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ResponseCode {
    NoError = 0,
    FormatError = 1,
    ServerFailure = 2,
    NameError = 3,
    NotImplemented = 4,
    Refused = 5,
}

impl From<u8> for ResponseCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::NoError,
            1 => Self::FormatError,
            2 => Self::ServerFailure,
            3 => Self::NameError,
            4 => Self::NotImplemented,
            5 => Self::Refused,
            _ => unimplemented!("Response code for {value} isn't implemented "),
        }
    }
}

pub type DnsMessageBytes = [u8; 12];

pub fn serialize_header(msg: &DnsHeader) -> [u8; 12] {
    let mut bytes = [0; 12];

    bytes[0] = (msg.id >> 8) as u8;
    bytes[1] = msg.id as u8;
    bytes[2] |= (msg.query as u8) << 7;
    bytes[2] |= (msg.op_code as u8) << 3;
    bytes[2] |= (msg.aa as u8) << 2;
    bytes[2] |= (msg.tc as u8) << 1;
    bytes[2] |= msg.rd as u8;

    bytes[3] |= (msg.ra as u8) << 7;
    bytes[3] |= msg.z << 4;
    bytes[3] |= msg.response_code as u8;

    let u16_to_double_u8 = |bytes: &mut [u8], i: usize, value: u16| {
        bytes[i] |= (value >> 8) as u8;
        bytes[i + 1] |= value as u8;
    };

    u16_to_double_u8(&mut bytes, 4, msg.questions_count);
    u16_to_double_u8(&mut bytes, 6, msg.answers_count);
    u16_to_double_u8(&mut bytes, 8, msg.authority_count);
    u16_to_double_u8(&mut bytes, 10, msg.additional_count);

    bytes
}

pub fn deserialize_header(msg_bytes: &[u8]) -> DnsHeader {
    let id: u16 = (msg_bytes[1] as u16) | ((msg_bytes[0] as u16) << 8);
    let query = msg_bytes[2] & 0b10000000;
    let op_code = (msg_bytes[2] & 0b01111000) >> 3;
    let aa = msg_bytes[2] & 0b00000100 != 0;
    let tc = msg_bytes[2] & 0b00000010 != 0;
    let rd = msg_bytes[2] & 0b00000001 != 0;
    let ra = msg_bytes[3] & 0b10000000 != 0;
    let z = msg_bytes[3] & 0b01110000;
    let response_code = msg_bytes[3] & 0b00001111;

    // let double_u8_to_u16 = |bytes: &[u8], i: usize| (bytes[i] as u16) << 8 | (bytes[i + 1] as u16);

    let questions_count = utils::double_u8_to_u16(msg_bytes, 4);
    let answers_count = utils::double_u8_to_u16(msg_bytes, 6);
    let authority_count = utils::double_u8_to_u16(msg_bytes, 8);
    let additional_count = utils::double_u8_to_u16(msg_bytes, 10);

    DnsHeader {
        id,
        query: query.into(),
        op_code: op_code.into(),
        aa,
        tc,
        rd,
        ra,
        z,
        response_code: response_code.into(),
        questions_count,
        answers_count,
        authority_count,
        additional_count,
    }
}

#[test]
fn test_serialize_deserialize() {
    let msg = DnsHeader {
        id: 1234,
        query: QR::Response,
        op_code: OpCode::StandardQuery,
        aa: false,
        tc: false,
        rd: false,
        ra: false,
        z: 0,
        response_code: ResponseCode::NoError,
        questions_count: 0,
        answers_count: 0,
        authority_count: 0,
        additional_count: 0,
    };
    let my_res = serialize_header(&msg);

    let recv_msg = deserialize_header(&my_res);

    let response: [u8; 12] = [0x04, 0xd2, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    assert_eq!(response, my_res);

    assert_eq!(msg, recv_msg);

    let msg = DnsHeader {
        id: 1334,
        query: QR::Response,
        op_code: OpCode::StandardQuery,
        aa: false,
        tc: true,
        rd: false,
        ra: false,
        z: 0,
        response_code: ResponseCode::FormatError,
        questions_count: 2,
        answers_count: 1,
        authority_count: 1,
        additional_count: 1,
    };

    let my_res = serialize_header(&msg);

    eprintln!("res: {my_res:?}");

    let recv_msg = deserialize_header(&my_res);

    assert_eq!(msg, recv_msg);
}
