// use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, UdpSocket};

use dns_header::{deserialize_header, serialize_header, DnsHeader, OpCode, ResponseCode, QR};
use dns_record::{deserialize_record, serialize_record, DnsRecord};
mod dns_header;
mod dns_record;
mod utils;

#[derive(Debug, PartialEq, Eq)]
struct DnsMsg {
    header: DnsHeader,
    questions: Vec<DnsRecord>,
    answers: Vec<DnsRecord>,
    authority: Vec<DnsRecord>,
    additional: Vec<DnsRecord>,
}

fn serialize(msg: &DnsMsg) -> Vec<u8> {
    // let mut bytes = [0; 12];
    let mut bytes = serialize_header(&msg.header).to_vec();

    for record in &msg.questions {
        let record_bytes = serialize_record(record);
        bytes.extend_from_slice(&record_bytes);
    }

    for record in &msg.answers {
        let record_bytes = serialize_record(record);
        bytes.extend_from_slice(&record_bytes);
    }

    bytes
}

fn deserialize<'a>(msg_bytes: &'a [u8]) -> DnsMsg {
    let header = deserialize_header(msg_bytes);

    let size = msg_bytes.len();
    let bytes = &msg_bytes[12..size];
    println!("header:{header:?} msg: {:?}", &msg_bytes[12..size]);

    let deserialize_records = |n_records: u16, bytes: &'a [u8]| -> (Vec<DnsRecord>, &'a [u8]) {
        let mut records = Vec::with_capacity(n_records as usize);
        let mut index = 1;
        for _i in 0..n_records {
            let record;
            (record, index) = deserialize_record(bytes, index);
            println!(
                "record: {record:?} index: {index}, remain: {:?}",
                &bytes[index..]
            );
            records.push(record);
            // bytes = &bytes[index..]
        }

        (records, bytes)
    };

    let (questions, _bytes) = deserialize_records(header.questions_count, bytes);
    // let (answers, bytes) = deserialize_records(header.answers_count, bytes);
    // let (authority, bytes) = deserialize_records(header.authority_count, bytes);
    // let (additional, bytes) = deserialize_records(header.additional_count, bytes);
    println!("msg: {:?}", _bytes);

    DnsMsg {
        header,
        questions,
        answers: Vec::new(),
        authority: Vec::new(),
        additional: Vec::new(),
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                println!("Received all bytes {:?}", &buf[..size]);
                let mut dns_msg = deserialize(&buf[0..size]);
                println!("Received msg: {dns_msg:#?}");
                let mut answers = Vec::with_capacity(dns_msg.header.questions_count as usize);
                let default_anwser_ip = Ipv4Addr::new(8, 8, 8, 8);

                for record in &dns_msg.questions {
                    let ip_bytes = default_anwser_ip.octets();
                    let answer = DnsRecord {
                        name: record.name.clone(),
                        dns_type: record.dns_type,
                        dns_class: record.dns_class,
                        time_to_live: 60,
                        rd_length: ip_bytes.len() as u16,
                        rd_data: ip_bytes.to_vec(),
                    };

                    answers.push(answer);
                }
                dns_msg.header.query = QR::Response;
                dns_msg.header.answers_count = answers.len() as u16;
                dns_msg.answers = answers;
                if dns_msg.header.op_code != OpCode::StandardQuery {
                    dns_msg.header.response_code = ResponseCode::NotImplemented;
                }
                let response = serialize(&dns_msg);

                println!(
                    "response size {}, response bytes: {:?}",
                    response.len(),
                    response
                );
                udp_socket
                    .send_to(&response, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}

#[test]
fn test_serialize() {
    let request = [
        81, 180, 1, 0, 0, 2, 0, 0, 0, 0, 0, 0, 3, 97, 98, 99, 17, 108, 111, 110, 103, 97, 115, 115,
        100, 111, 109, 97, 105, 110, 110, 97, 109, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1, 3, 100,
        101, 102, 192, 16, 0, 1, 0, 1,
    ];

    let expected_msg = DnsMsg {
        header: DnsHeader {
            id: 20916,
            query: QR::Query,
            op_code: OpCode::StandardQuery,
            aa: false,
            tc: false,
            rd: true,
            ra: false,
            z: 0,
            response_code: ResponseCode::NoError,
            questions_count: 2,
            answers_count: 0,
            authority_count: 0,
            additional_count: 0,
        },
        questions: vec![
            DnsRecord {
                name: "abc.longassdomainname.com".into(),
                dns_type: dns_record::DnsType::A,
                dns_class: dns_record::DnsClass::IN,
                time_to_live: 0,
                rd_length: 0,
                rd_data: vec![],
            },
            DnsRecord {
                name: "def.longassdomainname.com".into(),
                dns_type: dns_record::DnsType::A,
                dns_class: dns_record::DnsClass::IN,
                time_to_live: 0,
                rd_length: 0,
                rd_data: vec![],
            },
        ],
        answers: vec![],
        authority: vec![],
        additional: vec![],
    };

    // let msg = deserialize(&request);
    // assert_eq!(expected_msg, msg);
    //
    let rest = serialize(&expected_msg);
    let result = [
        81, 180, 1, 0, 0, 2, 0, 0, 0, 0, 0, 0, 3, 97, 98, 99, 17, 108, 111, 110, 103, 97, 115, 115,
        100, 111, 109, 97, 105, 110, 110, 97, 109, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1, 3, 100,
        101, 102, 17, 108, 111, 110, 103, 97, 115, 115, 100, 111, 109, 97, 105, 110, 110, 97, 109,
        101, 3, 99, 111, 109, 0, 0, 1, 0, 1,
    ];
    let msg = deserialize(&result);
    assert_eq!(expected_msg, msg);
}
