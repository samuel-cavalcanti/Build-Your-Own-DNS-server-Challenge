// use serde::{Deserialize, Serialize};
use std::net::UdpSocket;

use dns_header::{deserialize_header, serialize_header, DnsHeader, DnsMessageBytes};
use dns_record::{deserialize_record, DnsRecord};
mod dns_header;
mod dns_record;

#[derive(Debug)]
struct DnsMsg {
    header: DnsHeader,
    questions: Vec<DnsRecord>,
    answers: Vec<DnsRecord>,
    authority: Vec<DnsRecord>,
    additional: Vec<DnsRecord>,
}



fn serialize(msg: &DnsMsg) -> DnsMessageBytes {
    serialize_header(&msg.header)
}



fn deserialize(msg_bytes: &[u8]) -> DnsMsg {
    let header = deserialize_header(msg_bytes);

    let size = msg_bytes.len();
    let bytes = &msg_bytes[12..size];

    for _i in 0..header.questions_count {
        let record = deserialize_record(&bytes);
    }

    println!("msg: {:?}", &msg_bytes[12..size]);

    DnsMsg {
        header,
        questions: Vec::new(),
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
                let _msg = DnsHeader {
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
                println!("Received {} bytes from {}", size, source);
                let received_msg = deserialize(&buf[0..size]);
                println!("Received msg: {received_msg:#?}");
                let response = serialize(&received_msg);

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

