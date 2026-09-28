use gem_encoding::protobuf::proto_decode;

#[derive(Clone, Debug, Default)]
pub struct Timestamp {
    pub seconds: i64,
    pub nanos: i32,
}

impl Timestamp {
    pub fn millis(&self) -> i64 {
        self.seconds.saturating_mul(1000) + i64::from(self.nanos / 1_000_000)
    }
}

proto_decode!(Timestamp {
    1 => seconds: varint_i64,
    2 => nanos: varint_i32,
});
