// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemAmountInput
import protocol Gemstone.GemAmountServiceProtocol
import enum Gemstone.GemAmountTransfer
import enum Gemstone.GemAmountType
import struct Gemstone.GemAssetBalance
import struct Gemstone.GemPaymentRecipient
import struct Gemstone.GemTransferData
import GemstonePrimitives
import Localization
import Primitives

public final class AmountTransferViewModel: AmountDataProvidable {
    let asset: Asset
    let transfer: GemAmountTransfer
    private let service: any GemAmountServiceProtocol

    init(asset: Asset, transfer: GemAmountTransfer, service: any GemAmountServiceProtocol) {
        self.asset = asset
        self.transfer = transfer
        self.service = service
    }

    var displayAsset: Asset {
        transfer.displayAsset(asset: asset.toGem()).toPrimitives()
    }

    var title: String {
        gemAmountType.title().title
    }

    var gemAmountType: GemAmountType {
        transfer.amountType()
    }

    func input(from assetData: AssetData) -> GemAmountInput {
        transfer.input(asset: asset.toGem(), balance: GemAssetBalance(assetData.balance, assetId: asset.id, isActive: assetData.metadata.isActive))
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) async throws -> GemTransferData {
        try await service.transferData(asset: asset.toGem(), transfer: transfer, value: value, useMaxAmount: useMaxAmount)
    }
}
