use gem_client::Target;

#[derive(Clone, Debug)]
pub enum PanoraTarget {
    Swap,
}

impl Target for PanoraTarget {
    fn path(&self) -> String {
        match self {
            Self::Swap => "/swap".to_string(),
        }
    }
}
