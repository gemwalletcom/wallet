// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
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

    init(asset: Asset, action: TransferAction) {
        self.asset = asset
        self.action = action
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

    var minimumValue: BigInt {
        switch action {
        case .send: .zero
        case .deposit: asset.symbol == "USDC" ? AmountPerpetualLimits.minDeposit : .zero
        case .withdraw: asset.symbol == "USDC" ? AmountPerpetualLimits.minWithdraw : .zero
        }
    }

    var canChangeValue: Bool { true }
    var reserveForFee: BigInt { .zero }

    func shouldReserveFee(from _: AssetData) -> Bool { false }

    func availableValue(from assetData: AssetData) -> BigInt {
        switch action {
        case .send, .deposit: assetData.balance.available
        case .withdraw: assetData.balance.withdrawable
        }
    }

    func recipientData() -> RecipientData {
        action.recipient
    }

    func makeTransferData(value: BigInt) throws -> TransferData {
        let transferType: TransferDataType = switch action {
        case .send: .transfer(asset)
        case .deposit: .deposit(asset)
        case .withdraw: .withdrawal(asset)
        }
        return TransferData(
            type: transferType,
            recipientData: action.recipient,
            value: value,
            canChangeValue: true,
        )
    }
}
