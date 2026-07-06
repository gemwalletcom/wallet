use primitives::Chain;

#[derive(Clone, Debug)]
pub struct AssetImage {
    pub chain: Chain,
    pub token_id: String,
    pub image_url: String,
}
