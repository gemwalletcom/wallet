// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemAssetBalance
import struct Gemstone.GemAmountInput
import enum Gemstone.GemAmountType
import GemstonePrimitives
import Primitives
import struct Gemstone.GemTransferData

protocol AmountDataProvidable {
    var asset: Asset { get }
    var title: String { get }
    var amountType: AmountType { get }
    var gemAmountType: GemAmountType { get }
    var prefilledAmount: String? { get }
    func makeTransferData(value: BigInt, useMaxAmount: Bool) async throws -> GemTransferData
}

extension AmountDataProvidable {
    var prefilledAmount: String? {
        nil
    }

    func input(from assetData: AssetData) -> GemAmountInput {
        gemAmountType.input(asset: asset.map(), balance: GemAssetBalance(assetData.balance, assetId: asset.id))
    }
}
