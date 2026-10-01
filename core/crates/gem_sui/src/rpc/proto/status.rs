use gem_encoding::protobuf::proto_decode;

#[derive(Clone, Debug, Default)]
pub struct Status {
    pub code: i32,
    pub message: String,
}

proto_decode!(Status {
    1 => code: varint_i32,
    2 => message: string,
});
