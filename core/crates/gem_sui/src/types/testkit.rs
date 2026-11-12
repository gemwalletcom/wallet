use super::{Address, Argument, Command, Input, MergeCoins, Mutability, ObjectInput, ProgrammableTransaction, SharedObject, SplitCoins};

impl ProgrammableTransaction {
    pub fn mock() -> Self {
        let amount = Input::Pure(1u64.to_le_bytes().to_vec());
        Self {
            inputs: vec![
                amount.clone(),
                amount,
                Input::Object(ObjectInput::Shared(SharedObject {
                    object_id: Address::ZERO,
                    initial_shared_version: 1,
                    mutability: Mutability::Mutable,
                })),
            ],
            commands: vec![
                Command::SplitCoins(SplitCoins {
                    coin: Argument::Gas,
                    amounts: vec![Argument::Input(0), Argument::Input(0), Argument::Input(1)],
                }),
                Command::MergeCoins(MergeCoins {
                    coin: Argument::Gas,
                    coins_to_merge: vec![Argument::NestedResult(0, 2), Argument::Input(2), Argument::Input(2), Argument::Input(2)],
                }),
            ],
        }
    }
}
