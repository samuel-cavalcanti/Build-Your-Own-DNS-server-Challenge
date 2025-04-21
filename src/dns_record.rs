#[derive(Debug)]
enum DnsType {
    A,
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

    // Types that appear only in question part of a query.
    Axfr,
    Mailb,
    Maila,
    AllRecords,
}

#[derive(Debug)]
enum DnsClass {
    IN,
    CS,
    CH,
    HS,
    // Any Class only appear in question section of a Query
    AnyClass,
}

#[derive(Debug)]
pub struct DnsRecord {
    pub name: String,
    pub dns_type: DnsType,
    pub dns_class: DnsClass,
    pub time_to_live: i32,
    pub rd_length: u16,
    pub rd_data: String,
}

pub fn deserialize_record(bytes: &[u8]) -> DnsRecord {
    let (mut begin, mut end, mut name) = deserialize_label(bytes, 1, bytes[0] as usize + 1);

    while begin != end {
        let label;
        (begin, end, label) = deserialize_label(bytes, begin, end);
        name = format!("{name}.{label}");
    }
    println!("end: {end}");
    let dns_type_byte = &bytes[end..end + 2];
    end += 2;
    let dns_class_byte = &bytes[end..end + 2];

    DnsRecord {
        name,
        dns_type: DnsType::A,
        dns_class: DnsClass::IN,
        time_to_live: 0,
        rd_length: 0,
        rd_data: "".to_string(),
    }
}

fn deserialize_label(bytes: &[u8], begin: usize, end: usize) -> (usize, usize, String) {
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
    let record = deserialize_record(&response);

    assert_eq!("codecrafters.io".to_string(), record.name);
    println!("domain: {record:?}");
}
