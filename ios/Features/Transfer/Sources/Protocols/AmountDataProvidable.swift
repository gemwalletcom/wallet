// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemAssetBalance
import struct Gemstone.GemAmountLimits
import struct Gemstone.GemAmountRules
import enum Gemstone.GemAmountType
import GemstonePrimitives
import Primitives
import struct Gemstone.GemTransferData

protocol AmountDataProvidable {
    var asset: Asset { get }
    var title: String { get }
    var amountType: AmountType { get }
    var gemAmountType: GemAmountType { get }
    func recipientData() -> RecipientData
    func makeTransferData(value: BigInt, useMaxAmount: Bool) async throws -> GemTransferData
}

extension AmountDataProvidable {
    var rules: GemAmountRules {
        gemAmountType.rules(asset: asset.map())
    }

    var reserveForFee: BigInt {
        rules.reserveForFee
    }

    var canChangeValue: Bool {
        rules.canChangeValue
    }

    var showsAssetBalance: Bool {
        rules.showsAssetBalance
    }

    func limits(from assetData: AssetData) -> GemAmountLimits {
        gemAmountType.limits(asset: asset.map(), balance: GemAssetBalance(assetData.balance, assetId: asset.id))
    }

    func availableValue(from assetData: AssetData) -> BigInt {
        limits(from: assetData).availableValue
    }

    func maxValue(from assetData: AssetData) -> BigInt {
        limits(from: assetData).maxValue
    }

    func shouldReserveFee(from assetData: AssetData) -> Bool {
        limits(from: assetData).reservesFee
    }
}
