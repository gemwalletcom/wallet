use std::fmt::Debug;
use std::hash::Hash;

use crate::decimal::{push_f64, push_u64};

const BUCKET_LABEL: &str = "le";
const INFINITE_BOUND: &str = "+Inf";

pub trait LabelSet: Clone + Debug + Eq + Hash + Send + Sync + 'static {
    fn encode(&self, encoder: &mut LabelEncoder<'_>);
}

pub trait LabelValue {
    fn encode(&self, out: &mut String);
}

pub struct LabelEncoder<'a> {
    out: &'a mut String,
    written: bool,
}

impl<'a> LabelEncoder<'a> {
    pub(crate) fn new(out: &'a mut String) -> Self {
        Self { out, written: false }
    }

    pub fn add<V: LabelValue + ?Sized>(&mut self, name: &str, value: &V) {
        self.push_name(name);
        value.encode(self.out);
        self.out.push('"');
    }

    pub(crate) fn add_bucket_bound(&mut self, bound: f64) {
        self.push_name(BUCKET_LABEL);
        if bound == f64::INFINITY {
            self.out.push_str(INFINITE_BOUND);
        } else {
            push_f64(self.out, bound);
        }
        self.out.push('"');
    }

    pub(crate) fn finish(self) {
        if self.written {
            self.out.push('}');
        }
    }

    fn push_name(&mut self, name: &str) {
        self.out.push(if self.written { ',' } else { '{' });
        self.written = true;
        self.out.push_str(name);
        self.out.push_str("=\"");
    }
}

impl LabelValue for str {
    fn encode(&self, out: &mut String) {
        let mut start = 0;
        for (index, byte) in self.bytes().enumerate() {
            let escaped = match byte {
                b'\\' => "\\\\",
                b'"' => "\\\"",
                b'\n' => "\\n",
                _ => continue,
            };
            out.push_str(&self[start..index]);
            out.push_str(escaped);
            start = index + 1;
        }
        out.push_str(&self[start..]);
    }
}

impl LabelValue for String {
    fn encode(&self, out: &mut String) {
        self.as_str().encode(out);
    }
}

impl LabelValue for u16 {
    fn encode(&self, out: &mut String) {
        push_u64(out, u64::from(*self));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_value(value: &str) -> String {
        let mut out = String::new();
        let mut encoder = LabelEncoder::new(&mut out);
        encoder.add("label", value);
        encoder.finish();
        out
    }

    #[test]
    fn test_add() {
        assert_eq!(encode_value("plain \\ quoted \" line\ncarriage\r unicode λ"), concat!(r#"{label="plain \\ quoted \" line\ncarriage"#, "\r", r#" unicode λ"}"#));
        assert_eq!(encode_value("a\"} evil{x=\"1\\line\nnext"), r#"{label="a\"} evil{x=\"1\\line\nnext"}"#);
        assert_eq!(encode_value("\\\"\n"), r#"{label="\\\"\n"}"#);
        assert_eq!(encode_value(""), r#"{label=""}"#);

        let mut out = String::new();
        let mut encoder = LabelEncoder::new(&mut out);
        encoder.add_bucket_bound(0.001);
        encoder.add("status", &404u16);
        encoder.add("route", &"/v1/{id}".to_string());
        encoder.finish();
        assert_eq!(out, r#"{le="0.001",status="404",route="/v1/{id}"}"#);

        let mut out = String::new();
        LabelEncoder::new(&mut out).finish();
        assert_eq!(out, "");
    }
}
