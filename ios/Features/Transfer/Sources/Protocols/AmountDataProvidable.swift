// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemAmountEntry
import struct Gemstone.GemAmountInput
import enum Gemstone.GemAmountInputType
import enum Gemstone.GemAmountType
import struct Gemstone.GemAssetBalance
import struct Gemstone.GemTransferData
import GemstonePrimitives
import Primitives

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

    func entry(from assetData: AssetData, inputType: GemAmountInputType, text: String) -> GemAmountEntry {
        gemAmountType.entry(
            asset: asset.map(),
            input: input(from: assetData),
            price: assetData.price?.price,
            inputType: inputType,
            text: text,
        )
    }
}
