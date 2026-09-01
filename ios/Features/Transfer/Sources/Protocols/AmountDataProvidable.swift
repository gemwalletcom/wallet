// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemTransferBalance
import enum Gemstone.GemAmountError
import struct Gemstone.GemAmountLimits
import struct Gemstone.GemAmountRules
import class Gemstone.GemAmountService
import enum Gemstone.GemAmountType
import GemstonePrimitives
import Primitives
import struct Gemstone.GemTransferData

protocol AmountDataProvidable {
    var asset: Asset { get }
    var amountService: GemAmountService { get }
    var title: String { get }
    var amountType: AmountType { get }
    var gemAmountType: GemAmountType { get }
    func recipientData() -> RecipientData
    func makeTransferData(value: BigInt, useMaxAmount: Bool) async throws -> GemTransferData
}

extension AmountDataProvidable {
    var rules: GemAmountRules {
        amountService.rules(amountType: gemAmountType, asset: asset.map())
    }

    var minimumValue: BigInt {
        BigInt(stringLiteral: rules.minimumValue)
    }

    var reserveForFee: BigInt {
        BigInt(stringLiteral: rules.reserveForFee)
    }

    var canChangeValue: Bool {
        rules.canChangeValue
    }

    var showsAssetBalance: Bool {
        rules.showsAssetBalance
    }

    func limits(from assetData: AssetData) -> GemAmountLimits {
        do {
            return try amountService.limits(amountType: gemAmountType, asset: asset.map(), balance: GemTransferBalance(assetData.balance))
        } catch let error as GemAmountError {
            debugLog("amount limits unavailable: \(error)")
            return GemAmountLimits(availableValue: "0", maxValue: "0", reservesFee: false)
        } catch {
            preconditionFailure("Unencodable amount asset: \(error)")
        }
    }

    func availableValue(from assetData: AssetData) -> BigInt {
        BigInt(stringLiteral: limits(from: assetData).availableValue)
    }

    func maxValue(from assetData: AssetData) -> BigInt {
        BigInt(stringLiteral: limits(from: assetData).maxValue)
    }

    func shouldReserveFee(from assetData: AssetData) -> Bool {
        limits(from: assetData).reservesFee
    }
}

private extension GemTransferBalance {
    init(_ balance: Balance) {
        self.init(
            available: balance.available.description,
            frozen: balance.frozen.description,
            locked: balance.locked.description,
            withdrawable: balance.withdrawable.description,
            votes: UInt32(balance.metadata?.votes ?? 0),
        )
    }
}
