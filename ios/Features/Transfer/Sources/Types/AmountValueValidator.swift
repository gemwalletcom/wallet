// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import enum Gemstone.GemAmountError
import class Gemstone.GemAmountService
import Primitives
import Validators

struct AmountValueValidator: ValueValidator {
    private let asset: Asset
    private let available: BigInt
    private let minimum: BigInt

    init(asset: Asset, available: BigInt, minimum: BigInt) {
        self.asset = asset
        self.available = available
        self.minimum = minimum
    }

    func validate(_ value: BigInt) throws {
        do {
            try GemAmountService().validate(value: value.description, availableValue: available.description, minimumValue: minimum.description)
        } catch GemAmountError.Zero {
            throw SilentValidationError()
        } catch GemAmountError.BelowMinimum {
            throw TransferError.minimumAmount(asset: asset, required: minimum)
        } catch GemAmountError.InsufficientBalance {
            throw TransferAmountCalculatorError.insufficientBalance(
                asset,
                requirement: BalanceRequirement(required: value, available: available),
            )
        }
    }

    var id: String {
        "AmountValueValidator<\(asset.symbol)>"
    }
}
