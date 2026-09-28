#[derive(Model)]
#[model(swift = "Equatable, Sendable")]
pub enum Currency {
    USD,
    EUR,
}
