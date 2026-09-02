// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import enum Gemstone.GemAmountError
import enum Gemstone.GemAmountType
import struct Gemstone.GemAssetBalance
import GemstonePrimitives
import Primitives
import Validators

struct AmountValueValidator: ValueValidator {
    private let type: GemAmountType
    private let asset: Asset
    private let balance: Balance

    init(type: GemAmountType, asset: Asset, balance: Balance) {
        self.type = type
        self.asset = asset
        self.balance = balance
    }

    func validate(_ value: BigInt) throws {
        do {
            try type.validate(asset: asset.map(), balance: GemAssetBalance(balance, assetId: asset.id), value: value.description)
        } catch GemAmountError.Zero {
            throw SilentValidationError()
        } catch GemAmountError.InvalidValue {
            throw TransferError.invalidAmount
        } catch let GemAmountError.BelowMinimum(minimum) {
            throw TransferError.minimumAmount(asset: asset, required: try BigInt.from(string: minimum))
        } catch let GemAmountError.InsufficientBalance(available) {
            throw TransferAmountCalculatorError.insufficientBalance(
                asset,
                requirement: BalanceRequirement(required: value, available: try BigInt.from(string: available)),
            )
        }
    }

    var id: String {
        "AmountValueValidator<\(asset.symbol)>"
    }
}
