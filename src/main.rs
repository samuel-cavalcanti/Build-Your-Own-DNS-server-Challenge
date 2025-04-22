use std::net::{SocketAddrV4, UdpSocket};

use dns_header::{deserialize_header, DnsHeader};
use dns_header::{serialize_header, QR};
use dns_record::{deserialize_record, serialize_record, DnsRecord};
mod dns_header;
mod dns_record;
mod utils;

#[derive(Debug, PartialEq, Eq, Clone)]
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

fn deserialize(msg_bytes: &[u8]) -> DnsMsg {
    let header = deserialize_header(msg_bytes);

    let bytes = msg_bytes;
    const HEADER_N_BYTES: usize = 12;

    let deserialize_records =
        |n_records: u16, index: usize, have_rd: bool| -> (Vec<DnsRecord>, usize) {
            let mut records = Vec::with_capacity(n_records as usize);
            let mut index = index;
            for _i in 0..n_records {
                let record;
                (record, index) = deserialize_record(bytes, index, have_rd);
                records.push(record);
                index += 1;
                // bytes = &bytes[index..]
            }

            (records, index)
        };

    let (questions, index) = deserialize_records(header.questions_count, HEADER_N_BYTES + 1, false);
    let (answers, index) = deserialize_records(header.answers_count, index, true);
    let (authority, index) = deserialize_records(header.authority_count, index, false);
    let (additional, _index) = deserialize_records(header.additional_count, index, false);

    DnsMsg {
        header,
        questions,
        answers,
        authority,
        additional,
    }
}

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// the socket <address> where <address> will be of the form <ip>:<port>
    #[arg(short, long)]
    resolver: SocketAddrV4,
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let args = Args::parse();

    println!("resolver {}", args.resolver);
    let resolver = std::net::SocketAddr::V4(args.resolver);

    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");

    let mut buf_client = [0; 512];
    let mut buff_resolver = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf_client) {
            Ok((size_client, source_client)) => {
                println!("Received {} bytes from {}", size_client, source_client);
                println!("Received all bytes {:?}", &buf_client[..size_client]);

                let client_msg = deserialize(&buf_client[0..size_client]);
                println!("Received msg: {client_msg:#?} from {source_client}");

                if client_msg.header.questions_count == 1 {
                    udp_socket
                        .send_to(&buf_client[0..size_client], resolver)
                        .expect("Failed to send buffer to resolver");

                    let (size_resolver, source_resolver) =
                        udp_socket.recv_from(&mut buff_resolver).unwrap();
                    assert_eq!(source_resolver, resolver);

                    udp_socket
                        .send_to(&buff_resolver[..size_resolver], source_client)
                        .expect("Failed to send response");
                } else {
                    let mut header = client_msg.header;
                    header.questions_count = 1;

                    let msgs: Vec<DnsMsg> = client_msg
                        .questions
                        .iter()
                        .map(|r| DnsMsg {
                            header,
                            questions: vec![r.clone()],
                            authority: vec![],
                            additional: vec![],
                            answers: vec![],
                        })
                        .collect();

                    let recv_msgs: Vec<DnsMsg> = msgs
                        .iter()
                        .map(|msg| {
                            udp_socket
                                .send_to(&serialize(msg), resolver)
                                .expect("Failed to send buffer to resolver");

                            let (size_resolver, source_resolver) =
                                udp_socket.recv_from(&mut buff_resolver).unwrap();
                            assert_eq!(source_resolver, resolver);
                            let resolver_msg = deserialize(&buff_resolver[0..size_resolver]);
                            println!("Received msg: {resolver_msg:#?} from {source_resolver}");

                            resolver_msg
                        })
                        .collect();

                    let mut answers =
                        Vec::with_capacity(client_msg.header.questions_count as usize);

                    for msg in recv_msgs {
                        answers.extend(msg.answers);
                    }

                    let mut client_msg = client_msg;

                    client_msg.header.query = QR::Response;
                    client_msg.answers = answers;
                    client_msg.header.answers_count = client_msg.header.questions_count;

                    udp_socket
                        .send_to(&serialize(&client_msg), source_client)
                        .expect("Failed to send response");
                }

                // let response_msg = handle_dns(dns_msg);
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        deserialize,
        dns_header::{DnsHeader, OpCode, ResponseCode, QR},
        dns_record::{self, DnsRecord},
        serialize, DnsMsg,
    };

    #[test]
    fn test_serialize() {
        let request = [
            81, 180, 1, 0, 0, 2, 0, 0, 0, 0, 0, 0, 3, 97, 98, 99, 17, 108, 111, 110, 103, 97, 115,
            115, 100, 111, 109, 97, 105, 110, 110, 97, 109, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1, 3,
            100, 101, 102, 192, 16, 0, 1, 0, 1,
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

        let msg = deserialize(&request);
        assert_eq!(expected_msg, msg);

        let msg_bytes = serialize(&expected_msg);
        let msg = deserialize(&msg_bytes);
        assert_eq!(expected_msg, msg);

        let result = [
            81, 180, 1, 0, 0, 2, 0, 0, 0, 0, 0, 0, 3, 97, 98, 99, 17, 108, 111, 110, 103, 97, 115,
            115, 100, 111, 109, 97, 105, 110, 110, 97, 109, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1, 3,
            100, 101, 102, 17, 108, 111, 110, 103, 97, 115, 115, 100, 111, 109, 97, 105, 110, 110,
            97, 109, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1,
        ];
        let msg = deserialize(&result);
        assert_eq!(expected_msg, msg);
    }

    #[test]
    fn test_deserialize() {
        let buffer = [
            16, 191, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 12, 99, 111, 100, 101, 99, 114, 97, 102, 116,
            101, 114, 115, 2, 105, 111, 0, 0, 1, 0, 1,
        ];

        let msg = deserialize(&buffer);

        let expected_msg = DnsMsg {
            header: DnsHeader {
                id: 4287,
                query: QR::Query,
                op_code: OpCode::StandardQuery,
                aa: false,
                tc: false,
                rd: true,
                ra: false,
                z: 0,
                response_code: ResponseCode::NoError,
                questions_count: 1,
                answers_count: 0,
                authority_count: 0,
                additional_count: 0,
            },
            questions: vec![DnsRecord {
                name: "codecrafters.io".into(),
                dns_type: dns_record::DnsType::A,
                dns_class: dns_record::DnsClass::IN,
                time_to_live: 0,
                rd_length: 0,
                rd_data: vec![],
            }],
            answers: vec![],
            authority: vec![],
            additional: vec![],
        };

        assert_eq!(msg, expected_msg);
    }
}
