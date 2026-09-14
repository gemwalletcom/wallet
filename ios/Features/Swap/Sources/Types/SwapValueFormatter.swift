// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import GemstonePrimitives
import Primitives

public struct SwapValueFormatter {
    private let formatter: ValueFormatter

    public init(valueFormatter: ValueFormatter) {
        formatter = valueFormatter
    }

    public func format(inputValue: String, decimals: Int) throws -> BigInt {
        let value = try NumberInput.value(inputValue, decimals: decimals)
        guard value > 0 else {
            throw SwapQuoteInputError.invalidAmount
        }
        return value
    }

    public func format(value: BigInt, decimals: Int) -> String {
        formatter.string(value, decimals: decimals)
    }
}
