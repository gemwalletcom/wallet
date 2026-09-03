use gem_encoding::protobuf::{proto_decode, proto_encode};

#[derive(Clone, Debug, Default)]
pub struct LookupNameRequest {
    pub name: Option<String>,
}

proto_encode!(LookupNameRequest {
    1 => name: optional_string,
});

#[derive(Clone, Debug, Default)]
pub struct LookupNameResponse {
    pub record: Option<NameRecord>,
}

proto_decode!(LookupNameResponse {
    1 => record: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct NameRecord {
    pub target_address: Option<String>,
}

proto_decode!(NameRecord {
    5 => target_address: optional_string,
});

#[cfg(test)]
mod tests {
    use gem_encoding::protobuf::{MessageDecode, MessageEncode, encode_bytes_field, encode_string_field};

    use super::{LookupNameRequest, LookupNameResponse};

    #[test]
    fn test_lookup_name_request_encode() {
        let request = LookupNameRequest {
            name: Some("alpha.sui".to_string()),
        };

        assert_eq!(request.encode(), encode_string_field(1, "alpha.sui"));
    }

    #[test]
    fn test_lookup_name_response_decode() {
        let target = "0x54e5c2a6f1276ac2ff623ac54e53e5a61a576906b3ec42fac8fe8bf5615d0957";
        let record = [
            encode_string_field(1, "record-id"),
            encode_string_field(2, "alpha.sui"),
            encode_string_field(5, target),
            encode_bytes_field(6, &[encode_string_field(1, "avatar"), encode_string_field(2, "ipfs://avatar")].concat()),
        ]
        .concat();

        let response = LookupNameResponse::decode(&encode_bytes_field(1, &record)).unwrap();

        assert_eq!(response.record.unwrap().target_address.as_deref(), Some(target));
    }
}
