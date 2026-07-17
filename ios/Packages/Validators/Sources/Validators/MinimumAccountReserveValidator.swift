// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives

struct MinimumAccountReserveValidator: ValueValidator {
    private let available: BigInt
    private let requiredReserve: BigInt
    private let asset: Asset

    init(available: BigInt, reserve: BigInt, asset: Asset) {
        self.available = available
        requiredReserve = reserve
        self.asset = asset
    }

    func validate(_ value: BigInt) throws {
        guard requiredReserve > 0, asset.type == .native else { return }

        let remaining = available - value

        if remaining.isBetween(1, and: requiredReserve) {
            throw TransferAmountCalculatorError.minimumAccountBalanceTooLow(
                asset,
                requirement: BalanceRequirement(required: requiredReserve, available: remaining),
            )
        }
    }

    var id: String {
        "MinAccountReserveValidator<\(asset.symbol)>"
    }
}
