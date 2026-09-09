use primitives::Chain;

#[derive(Clone, Debug, PartialEq)]
pub struct AssetImage {
    pub chain: Chain,
    pub token_id: String,
    pub image_url: String,
}
