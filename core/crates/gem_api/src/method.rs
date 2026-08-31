use gem_jsonrpc::HttpMethod;
use strum::AsRefStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, AsRefStr)]
#[strum(serialize_all = "UPPERCASE")]
pub enum GemApiMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl From<GemApiMethod> for HttpMethod {
    fn from(method: GemApiMethod) -> Self {
        match method {
            GemApiMethod::Get => HttpMethod::Get,
            GemApiMethod::Post => HttpMethod::Post,
            GemApiMethod::Put => HttpMethod::Put,
            GemApiMethod::Delete => HttpMethod::Delete,
        }
    }
}
