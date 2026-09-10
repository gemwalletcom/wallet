// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import protocol Gemstone.GemAmountServiceProtocol
import enum Gemstone.GemAmountTransfer
import enum Gemstone.GemAmountType
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
        transfer.displayAsset(asset: asset.map()).map()
    }

    var title: String {
        switch transfer {
        case .send: Localized.Transfer.Send.title
        case .deposit: Localized.Wallet.deposit
        case .withdraw: Localized.Wallet.withdraw
        }
    }

    var amountType: AmountType {
        switch transfer {
        case let .send(payment): .transfer(recipient: payment)
        case .deposit: .deposit
        case .withdraw: .withdraw
        }
    }

    var gemAmountType: GemAmountType {
        transfer.amountType()
    }

    var prefilledAmount: String? {
        transfer.prefilledAmount()
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) async throws -> GemTransferData {
        try await service.transferData(asset: asset.map(), transfer: transfer, value: value, useMaxAmount: useMaxAmount)
    }
}
