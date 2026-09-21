use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Eq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub struct BlockExplorerLink {
    pub name: String,
    pub link: String,
}
