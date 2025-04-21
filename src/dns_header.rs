#[derive(Debug, PartialEq)]
pub struct DnsHeader {
    pub id: u16,
    pub query: bool,
    pub op_code: u8,
    pub aa: bool,
    pub tc: bool,
    pub rd: bool,
    pub ra: bool,
    pub z: u8,
    pub response_code: u8,
    pub questions_count: u16,
    pub answers_count: u16,
    pub authority_count: u16,
    pub additional_count: u16,
}

pub type DnsMessageBytes = [u8; 12];

pub fn serialize_header(msg: &DnsHeader) -> [u8; 12] {
    let mut bytes = [0; 12];

    bytes[0] = (msg.id >> 8) as u8;
    bytes[1] = msg.id as u8;
    bytes[2] |= (msg.query as u8) << 7;
    bytes[2] |= msg.op_code << 3;
    bytes[2] |= (msg.aa as u8) << 2;
    bytes[2] |= (msg.tc as u8) << 1;
    bytes[2] |= msg.rd as u8;

    bytes[3] |= (msg.ra as u8) << 7;
    bytes[3] |= msg.z << 4;
    bytes[3] |= msg.response_code;

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
    let query = msg_bytes[2] & 0b10000000 != 0;
    let op_code = msg_bytes[2] & 0b01111000;
    let aa = msg_bytes[2] & 0b00000100 != 0;
    let tc = msg_bytes[2] & 0b00000010 != 0;
    let rd = msg_bytes[2] & 0b00000001 != 0;
    let ra = msg_bytes[3] & 0b10000000 != 0;
    let z = msg_bytes[3] & 0b01110000;
    let response_code = msg_bytes[3] & 0b00001111;

    let double_u8_to_u16 = |bytes: &[u8], i: usize| (bytes[i] as u16) << 8 | (bytes[i + 1] as u16);

    let questions_count = double_u8_to_u16(msg_bytes, 4);
    let answers_count = double_u8_to_u16(msg_bytes, 6);
    let authority_count = double_u8_to_u16(msg_bytes, 8);
    let additional_count = double_u8_to_u16(msg_bytes, 10);

    DnsHeader {
        id,
        query,
        op_code,
        aa,
        tc,
        rd,
        ra,
        z,
        response_code,
        questions_count,
        answers_count,
        authority_count,
        additional_count,
    }
}

#[test]
fn test_serialize_deserialize() {
    let response: [u8; 12] = [0x04, 0xd2, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let msg = DnsHeader {
        id: 1234,
        query: true,
        op_code: 0,
        aa: false,
        tc: false,
        rd: false,
        ra: false,
        z: 0,
        response_code: 0,
        questions_count: 0,
        answers_count: 0,
        authority_count: 0,
        additional_count: 0,
    };
    let my_res = serialize_header(&msg);

    let recv_msg = deserialize_header(&my_res);

    assert_eq!(response, my_res);
    assert_eq!(msg, recv_msg);

    let msg = DnsHeader {
        id: 1334,
        query: true,
        op_code: 0,
        aa: false,
        tc: true,
        rd: false,
        ra: false,
        z: 0,
        response_code: 1,
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
