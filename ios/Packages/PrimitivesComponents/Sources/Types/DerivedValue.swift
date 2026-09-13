// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

@MainActor
public final class DerivedValue<Input: Equatable, Value> {
    private var current: (input: Input, value: Value)?

    public init() {}

    public func callAsFunction(_ input: Input, _ derive: (Input) -> Value) -> Value {
        if let current, current.input == input {
            return current.value
        }
        let value = derive(input)
        current = (input, value)
        return value
    }
}
