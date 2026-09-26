use model_derive::Model;

#[derive(Debug, Clone, PartialEq, Eq, Model)]
#[model(swift = "Equatable, Hashable, Sendable")]
pub struct BlockExplorerLink {
    pub name: String,
    pub link: String,
}
