// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemTransactionInputType
import class Gemstone.GemAmountService
import BigInt
import Foundation
import enum Gemstone.GemAmountType
import GemstonePrimitives
import Localization
import Primitives

enum TransferAction {
    case send(RecipientData)
    case deposit(RecipientData)
    case withdraw(RecipientData)

    var recipient: RecipientData {
        switch self {
        case let .send(data), let .deposit(data), let .withdraw(data):
            data
        }
    }
}

public final class AmountTransferViewModel: AmountDataProvidable {
    let asset: Asset
    let action: TransferAction
    let amountService: GemAmountService

    init(asset: Asset, action: TransferAction, amountService: GemAmountService) {
        self.amountService = amountService
        self.asset = asset
        self.action = action
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
        case let .deposit(recipient): .deposit(recipient: recipient)
        case let .withdraw(recipient): .withdraw(recipient: recipient)
        }
    }

    var gemAmountType: GemAmountType {
        switch action {
        case .send: .transfer
        case .deposit: .deposit
        case .withdraw: .withdraw
        }
    }

    func recipientData() -> RecipientData {
        action.recipient
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) throws -> TransferData {
        let transferType: GemTransactionInputType = switch action {
        case .send: .transfer(asset)
        case .deposit: .deposit(asset)
        case .withdraw: .withdrawal(asset)
        }
        return TransferData(
            type: transferType,
            recipient: action.recipient.recipient,
            value: value,
            useMaxAmount: useMaxAmount,
        )
    }
}
