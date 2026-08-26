use strum::AsRefStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, AsRefStr)]
#[strum(serialize_all = "UPPERCASE")]
pub enum GemApiMethod {
    Get,
    Post,
}
