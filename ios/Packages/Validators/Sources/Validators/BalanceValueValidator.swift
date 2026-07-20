// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives

public struct BalanceValueValidator: ValueValidator {
    private let available: BigInt
    private let asset: Asset

    public init(available: BigInt, asset: Asset) {
        self.available = available
        self.asset = asset
    }

    public func validate(_ value: BigInt) throws {
        guard value <= available else {
            throw TransferAmountCalculatorError.insufficientBalance(
                asset,
                requirement: BalanceRequirement(required: value, available: available),
            )
        }
    }

    public var id: String {
        "BalanceValidator<\(asset.symbol)>"
    }
}
