// use serde::{Deserialize, Serialize};
use std::net::UdpSocket;

#[derive(Debug)]
struct DsnMsg {
    id: i16,
    query: bool,
    op_code: u8,
    aa: bool,
    tc: bool,
    rd: bool,
    ra: bool,
    z: u8,
    response_code: u8,
    questions_count: u16,
    answers_count: u16,
    authority_count: u16,
    additional_count: u16,
}

type DnsMessageBytes = [u8; 12];

fn serialize(msg: &DsnMsg) -> DnsMessageBytes {
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


    bytes[4] |= (msg.questions_count >> 8) as u8;
    bytes[5] |= msg.questions_count as u8;

    bytes[6] |= (msg.answers_count >> 8) as u8;
    bytes[7] |= msg.answers_count as u8;

    bytes[8] |= (msg.authority_count >> 8) as u8;
    bytes[9] |= msg.authority_count as u8;

    bytes[10] |= (msg.additional_count >> 8) as u8;
    bytes[11] |= msg.additional_count as u8;

    bytes
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
                let msg = DsnMsg {
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
                let response = serialize(&msg);
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
    let response: [u8; 12] = [0x04, 0xd2, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let msg = DsnMsg {
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
    let my_res = serialize(&msg);

    assert_eq!(response, my_res);
}
