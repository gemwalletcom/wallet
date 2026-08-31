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
    private let amountService: GemAmountService

    init(asset: Asset, available: BigInt, minimum: BigInt, amountService: GemAmountService) {
        self.asset = asset
        self.available = available
        self.minimum = minimum
        self.amountService = amountService
    }

    func validate(_ value: BigInt) throws {
        do {
            try amountService.validate(value: value.description, availableValue: available.description, minimumValue: minimum.description)
        } catch GemAmountError.Zero {
            throw SilentValidationError()
        } catch GemAmountError.InvalidValue {
            throw TransferError.invalidAmount
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
