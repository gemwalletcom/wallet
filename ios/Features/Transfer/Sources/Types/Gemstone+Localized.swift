// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import struct Gemstone.Asset
import enum Gemstone.GemAmountError
import enum Gemstone.GemAmountTitle
import enum Gemstone.GemConfirmButtonKind
import enum Gemstone.GemConfirmDestination
import enum Gemstone.GemRecipientSection
import enum Gemstone.GemConfirmError
import enum Gemstone.GemConfirmErrorDisplay
import enum Gemstone.GemConfirmTitle
import enum Gemstone.GemReceiveWarning
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
        case let .perpetualOpen(direction): PerpetualDirectionViewModel(direction: direction.toPrimitives()).title
        case let .perpetualIncrease(direction): PerpetualDirectionViewModel(direction: direction.toPrimitives()).increaseTitle
        case let .perpetualReduce(direction): PerpetualDirectionViewModel(direction: direction.toPrimitives()).reduceTitle
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
        case let .belowMinimum(asset, minimum):
            Localized.Transfer.minimumAmount(ValueFormatter(style: .auto).string(minimum, asset: asset.toPrimitives()).boldMarkdown())
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
        case let .balanceRequired(asset, requirement):
            Localized.Info.balanceRequiredDescription(
                Self.amount(requirement.required, asset: asset).boldMarkdown(),
                Self.amount(requirement.available, asset: asset).boldMarkdown(),
                Self.amount(requirement.shortfall, asset: asset).boldMarkdown(),
            )
        case let .networkFeeRequired(asset, requirement):
            Localized.Info.InsufficientNetworkFeeBalance.description(
                Self.amount(requirement.required, asset: asset).boldMarkdown(),
                asset.toPrimitives().chain.networkName.boldMarkdown(),
                Self.amount(requirement.available, asset: asset).boldMarkdown(),
                Self.amount(requirement.shortfall, asset: asset).boldMarkdown(),
            )
        case let .networkFeeMissing(asset):
            Localized.Transfer.insufficientNetworkFeeBalance(Self.title(asset: asset))
        case let .minimumAccountBalance(asset, required):
            Localized.Transfer.minimumAccountBalance(Self.amount(required, asset: asset).boldMarkdown())
        case let .swapMinimum(asset, _, providerName, requirement):
            Localized.Info.swapMinimumAmountDescription(
                providerName.boldMarkdown(),
                Self.amount(requirement.required, asset: asset).boldMarkdown(),
                Self.amount(requirement.available, asset: asset).boldMarkdown(),
                Self.amount(requirement.shortfall, asset: asset).boldMarkdown(),
            )
        case .dustThreshold: Localized.Errors.dustThresholdShort
        case .insufficientFunds: Localized.Info.InsufficientBalance.title
        case let .message(msg): msg
        }
    }

    private static func amount(_ value: BigInt, asset: Gemstone.Asset) -> String {
        ValueFormatter(style: .full).string(value, asset: asset.toPrimitives())
    }

    private static func title(asset: Gemstone.Asset) -> String {
        let title = asset.name == asset.symbol ? asset.name : String(format: "%@ (%@)", asset.name, asset.symbol)
        return title.boldMarkdown()
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

extension GemRecipientSection {
    var title: String {
        switch self {
        case .pinned: Localized.Common.pinned
        case .contacts: Localized.Contacts.title
        case .wallets: Localized.Transfer.Recipient.myWallets
        case .viewWallets: Localized.Transfer.Recipient.viewWallets
        }
    }
}
