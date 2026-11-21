use crate::labels::{LabelEncoder, LabelSet};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct LabelPairs(Vec<(&'static str, String)>);

impl LabelPairs {
    pub fn new(pairs: &[(&'static str, &str)]) -> Self {
        Self(pairs.iter().map(|(name, value)| (*name, value.to_string())).collect())
    }
}

impl LabelSet for LabelPairs {
    fn encode(&self, encoder: &mut LabelEncoder<'_>) {
        for (name, value) in &self.0 {
            encoder.add(name, value);
        }
    }
}
