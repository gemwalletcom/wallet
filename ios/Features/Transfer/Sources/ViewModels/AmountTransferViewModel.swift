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

enum TransferAction {
    case send(GemPaymentRecipient)
    case deposit
    case withdraw
}

public final class AmountTransferViewModel: AmountDataProvidable {
    let asset: Asset
    let action: TransferAction
    private let service: any GemAmountServiceProtocol

    init(asset: Asset, action: TransferAction, service: any GemAmountServiceProtocol) {
        self.asset = asset
        self.action = action
        self.service = service
    }

    var displayAsset: Asset {
        switch action {
        case .withdraw: PerpetualConfig.depositAsset
        case .send, .deposit: asset
        }
    }

    var title: String {
        switch action {
        case .send: Localized.Transfer.Send.title
        case .deposit: Localized.Wallet.deposit
        case .withdraw: Localized.Wallet.withdraw
        }
    }

    var amountType: AmountType {
        switch action {
        case let .send(recipient): .transfer(recipient: recipient)
        case .deposit: .deposit
        case .withdraw: .withdraw
        }
    }

    var gemAmountType: GemAmountType {
        switch action {
        case .send: .transfer
        case .deposit: .deposit
        case .withdraw: .withdraw
        }
    }

    var prefilledAmount: String? {
        guard case let .send(recipient) = action else { return nil }
        return recipient.amount
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) async throws -> GemTransferData {
        let transfer: GemAmountTransfer = switch action {
        case let .send(recipient): .send(recipient: recipient.recipient)
        case .deposit: .deposit
        case .withdraw: .withdraw
        }
        return try await service.transferData(asset: asset.map(), transfer: transfer, value: value, useMaxAmount: useMaxAmount)
    }
}
