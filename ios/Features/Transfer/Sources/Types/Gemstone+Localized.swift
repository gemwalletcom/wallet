// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemAmountError
import enum Gemstone.GemAmountTitle
import enum Gemstone.GemConfirmButtonKind
import enum Gemstone.GemConfirmDestination
import enum Gemstone.GemConfirmError
import enum Gemstone.GemConfirmErrorDisplay
import enum Gemstone.GemConfirmTitle
import enum Gemstone.GemReceiveWarning
import enum Gemstone.GemRecipientSectionKind
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

extension GemAmountTitle {
    var title: String {
        switch self {
        case .send: Localized.Transfer.Send.title
        case .deposit: Localized.Wallet.deposit
        case .withdraw: Localized.Wallet.withdraw
        case .stake: Localized.Transfer.Stake.title
        case .unstake: Localized.Transfer.Unstake.title
        case .redelegate: Localized.Transfer.Redelegate.title
        case .rewards: Localized.Transfer.ClaimRewards.title
        case .freeze: Localized.Transfer.Freeze.title
        case .unfreeze: Localized.Transfer.Unfreeze.title
        case let .perpetualOpen(direction): direction.toPrimitives().title
        case let .perpetualIncrease(direction): Localized.Perpetual.increaseDirection(direction.toPrimitives().title)
        case let .perpetualReduce(direction): Localized.Perpetual.reduceDirection(direction.toPrimitives().title)
        }
    }
}

extension GemConfirmButtonKind {
    var title: String {
        switch self {
        case .confirm: Localized.Transfer.confirm
        case .retry: Localized.Common.tryAgain
        case .accountMissing: Localized.Errors.walletAccountMissing
        }
    }
}

extension GemConfirmTitle {
    var title: String {
        switch self {
        case .send: Localized.Transfer.Send.title
        case .deposit: Localized.Wallet.deposit
        case .withdraw: Localized.Transfer.Withdraw.title
        case .swap: Localized.Wallet.swap
        case .approve: Localized.Transfer.Approve.title
        case .request: Localized.Transfer.reviewRequest
        case .payment: Localized.Transfer.paymentTitle
        case .stake: Localized.Transfer.Stake.title
        case .unstake: Localized.Transfer.Unstake.title
        case .redelegate: Localized.Transfer.Redelegate.title
        case .claimRewards: Localized.Transfer.ClaimRewards.title
        case .freeze: Localized.Transfer.Freeze.title
        case .unfreeze: Localized.Transfer.Unfreeze.title
        case .activateAsset: Localized.Transfer.ActivateAsset.title
        case let .perpetualOpen(direction): direction.toPrimitives().title
        case let .perpetualIncrease(direction): direction.toPrimitives().increaseTitle
        case let .perpetualReduce(direction): direction.toPrimitives().reduceTitle
        case .perpetualClose: Localized.Perpetual.closePosition
        case .perpetualModify: Localized.Perpetual.modifyPosition
        }
    }
}

extension GemAmountError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch display() {
        case .none: nil
        case .invalidAmount: Localized.Errors.invalidAmount
        case let .belowMinimum(minimum, _):
            Localized.Transfer.minimumAmount(minimum.text().boldMarkdown())
        case let .insufficientBalance(title):
            Localized.Transfer.insufficientBalance(title.boldMarkdown())
        }
    }
}

extension GemConfirmError: @retroactive LocalizedError {
    public var errorDescription: String? { display().errorDescription }
}

extension GemConfirmErrorDisplay: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .offline: Localized.Errors.networkOffline
        case .malicious: Localized.Errors.ScanTransaction.Malicious.description
        case let .memoRequired(symbol): Localized.Errors.ScanTransaction.memoRequired(symbol.boldMarkdown())
        case .feeRatesMissing: Localized.Errors.unableEstimateNetworkFee
        case .cancelled: Localized.Errors.cancelled
        case .accountMissing: Localized.Errors.walletAccountMissing
        case .unknown: Localized.Errors.unknown
        case let .balanceRequired(_, requirement):
            Localized.Info.balanceRequiredDescription(
                requirement.required.text().boldMarkdown(),
                requirement.available.text().boldMarkdown(),
                requirement.shortfall.text().boldMarkdown(),
            )
        case let .networkFeeRequired(asset, _, requirement):
            Localized.Info.InsufficientNetworkFeeBalance.description(
                requirement.required.text().boldMarkdown(),
                asset.toPrimitives().chain.networkName.boldMarkdown(),
                requirement.available.text().boldMarkdown(),
                requirement.shortfall.text().boldMarkdown(),
            )
        case let .networkFeeMissing(_, title):
            Localized.Transfer.insufficientNetworkFeeBalance(title.boldMarkdown())
        case let .minimumAccountBalance(_, required):
            Localized.Transfer.minimumAccountBalance(required.text().boldMarkdown())
        case let .destinationAccountActivation(_, required):
            Localized.Transfer.destinationAccountActivation(required.text().boldMarkdown())
        case let .swapMinimum(_, _, providerName, requirement):
            Localized.Info.swapMinimumAmountDescription(
                providerName.boldMarkdown(),
                requirement.required.text().boldMarkdown(),
                requirement.available.text().boldMarkdown(),
                requirement.shortfall.text().boldMarkdown(),
            )
        case .dustThreshold: Localized.Errors.dustThresholdShort
        case .insufficientFunds: Localized.Info.InsufficientBalance.title
        case let .payment(status): status.errorText
        case let .message(msg): msg
        }
    }
}

extension GemReceiveWarning {
    func text(asset: AssetViewModel) -> String {
        switch self {
        case .assetNetwork: Localized.Receive.warning(asset.symbol.boldMarkdown(), asset.networkFullName.boldMarkdown())
        case .noDestinationTagRequired: Localized.Wallet.Receive.noDestinationTagRequired
        case .noMemoRequired: Localized.Wallet.Receive.noMemoRequired
        }
    }
}

extension GemConfirmDestination {
    var title: String {
        switch self {
        case .recipient: Localized.Transfer.Recipient.title
        case .contract: Localized.Asset.contract
        case .validator: Localized.Stake.validator
        case .resource: Localized.Stake.resource
        case .provider: Localized.Common.provider
        }
    }
}

extension GemRecipientSectionKind {
    var title: String {
        switch self {
        case .pinned: Localized.Common.pinned
        case .contacts: Localized.Contacts.title
        case .wallets: Localized.Transfer.Recipient.myWallets
        case .viewWallets: Localized.Transfer.Recipient.viewWallets
        }
    }
}
