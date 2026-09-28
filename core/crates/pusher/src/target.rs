use gem_client::Target;

#[derive(Clone, Debug)]
pub enum PusherTarget {
    Push,
}

impl Target for PusherTarget {
    fn path(&self) -> String {
        match self {
            Self::Push => "/api/push".to_string(),
        }
    }
}
