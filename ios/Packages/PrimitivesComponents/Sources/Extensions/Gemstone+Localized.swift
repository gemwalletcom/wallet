// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import struct Gemstone.GemBannerAmount
import enum Gemstone.GemBannerTitle
import enum Gemstone.GemBannerDescription
import enum Gemstone.PerpetualDirection
import enum Gemstone.FeeOption
import enum Gemstone.GemAssetMenuAction
import enum Gemstone.GemContactAddressField
import enum Gemstone.PerpetualType
import enum Gemstone.GemApprovalValue
import enum Gemstone.GemAssetInfoKind
import enum Gemstone.GemEmptyStateAction
import enum Gemstone.GemErrorText
import enum Gemstone.GemSelectAssetSection
import enum Gemstone.GemSelectAssetTitle
import enum Gemstone.GemEmptyStateText
import enum Gemstone.GemFiatTransactionBadge
import enum Gemstone.LinkType
import enum Gemstone.GemHeaderButtonKind
import enum Gemstone.GemLocalizedText
import enum Gemstone.PaymentStatus
import enum Gemstone.GemPriceAlertLabel
import struct Gemstone.GemPriceAlertRow
import enum Gemstone.GemPriceAlertText
import enum Gemstone.GemRecipientErrorDisplay
import enum Gemstone.GemSimulationWarningKind
import enum Gemstone.SimulationPayloadFieldKind
import enum Gemstone.GemTransactionRowSubtitle
import enum Gemstone.GemTransactionStateTone
import enum Gemstone.GemTransactionTitle
import enum Gemstone.GemWalletSubtitle
import GemstonePrimitives
import enum Gemstone.GemDayLabel
import Localization
import Primitives
import Style
import SwiftUI


extension FeeOption {
    public var title: String {
        switch self {
        case .tokenAccountCreation: Localized.Banner.AccountActivation.title
        }
    }
}

extension GemLocalizedText {
    public var text: String {
        switch self {
        case let .walletDefaultName(index):
            Localized.Wallet.defaultName(Int(index))
        case let .walletDefaultNameChain(chain, index):
            Localized.Wallet.defaultNameChain(Chain(core: chain).networkName, Int(index))
        }
    }
}

extension GemPriceAlertText {
    public var text: String {
        switch self {
        case .empty: Placeholder.empty
        case let .number(value): value.text()
        case let .label(label): label.text
        }
    }
}

extension GemPriceAlertRow {
    public var prefixText: String {
        prefix.text
    }

    public var suffixText: String {
        suffix.text
    }
}

extension GemPriceAlertLabel {
    public var text: String {
        switch self {
        case .over: Localized.PriceAlerts.Direction.over
        case .under: Localized.PriceAlerts.Direction.under
        case .increasesBy: Localized.PriceAlerts.Direction.increasesBy
        case .decreasesBy: Localized.PriceAlerts.Direction.decreasesBy
        }
    }
}

extension GemTransactionTitle {
    public var title: String {
        switch self {
        case .received: Localized.Transaction.Title.received
        case .sent: Localized.Transaction.Title.sent
        case .transfer: Localized.Transfer.title
        case .smartContract: Localized.Transfer.SmartContract.title
        case .swap: Localized.Wallet.swap
        case .approve: Localized.Transfer.Approve.title
        case .stake: Localized.Transfer.Stake.title
        case .unstake: Localized.Transfer.Unstake.title
        case .redelegate: Localized.Transfer.Redelegate.title
        case .rewards: Localized.Transfer.Rewards.title
        case .withdraw: Localized.Transfer.Withdraw.title
        case .activateAsset: Localized.Transfer.ActivateAsset.title
        case .freeze: Localized.Transfer.Freeze.title
        case .unfreeze: Localized.Transfer.Unfreeze.title
        case .earn: Localized.Common.earn
        case let .perpetualOpen(direction):
            Self.perpetualTitle(direction, Localized.Perpetual.openDirection, Localized.Perpetual.position)
        case let .perpetualClose(direction):
            Self.perpetualTitle(direction, Localized.Perpetual.closeDirection, Localized.Perpetual.closePosition)
        case .perpetualModify: Localized.Perpetual.modify
        }
    }

    private static func perpetualTitle(
        _ direction: Gemstone.PerpetualDirection?,
        _ directionTitle: (String) -> String,
        _ fallback: String,
    ) -> String {
        guard let direction else { return fallback }
        return directionTitle(direction.toPrimitives().title)
    }
}

extension GemWalletSubtitle {
    public var text: String {
        switch self {
        case .multicoin: Localized.Wallet.multicoin
        case let .address(value): value
        }
    }
}

extension GemSimulationWarningKind {
    var warningTitle: String {
        switch self {
        case .unlimitedApproval: Localized.Simulation.Warning.UnlimitedTokenApproval.title
        case .nftCollectionApproval: Localized.Simulation.Warning.NftCollectionApproval.title
        case .externallyOwnedSpender: Localized.Common.warning
        case .suspiciousSpender, .validationError: Localized.Errors.errorOccurred
        }
    }

    var defaultMessage: String? {
        switch self {
        case .unlimitedApproval: Localized.Simulation.Warning.UnlimitedTokenApproval.description
        case .validationError: Localized.Errors.errorOccurred
        case .externallyOwnedSpender: Localized.Simulation.warningExternallyOwnedSpenderDescription
        case .suspiciousSpender: Localized.Common.suspiciousAddress
        case .nftCollectionApproval: nil
        }
    }
}

extension GemFiatTransactionBadge {
    public var text: String {
        switch self {
        case .pending: Localized.Transaction.Status.pending
        case .failed: Localized.Transaction.Status.failed
        }
    }
}

extension GemHeaderButtonKind {
    public var title: String {
        switch self {
        case .send: Localized.Wallet.send
        case .receive: Localized.Wallet.receive
        case .buy: Localized.Wallet.buy
        case .swap: Localized.Wallet.swap
        case .deposit: Localized.Wallet.deposit
        case .withdraw: Localized.Wallet.withdraw
        case .more: Localized.Wallet.more
        }
    }
}

extension GemAssetInfoKind {
    public var title: String {
        switch self {
        case .name: Localized.Asset.name
        case .symbol: Localized.Asset.symbol
        case .decimals: Localized.Asset.decimals
        case .kind: Localized.Common.type
        }
    }
}

extension ChartPeriod {
    public var title: String {
        switch self {
        case .hour: Localized.Charts.hour
        case .day: Localized.Charts.day
        case .week: Localized.Charts.week
        case .month: Localized.Charts.month
        case .year: Localized.Charts.year
        case .all: Localized.Charts.all
        }
    }
}

extension LinkType {
    public var title: String {
        switch self {
        case .x: Localized.Social.x
        case .discord: Localized.Social.discord
        case .reddit: Localized.Social.reddit
        case .telegram: Localized.Social.telegram
        case .gitHub: Localized.Social.github
        case .youTube: Localized.Social.youtube
        case .facebook: Localized.Social.facebook
        case .website: Localized.Social.website
        case .coingecko: Localized.Social.coingecko
        case .coinMarketCap: Localized.Social.coinmarketcap
        case .openSea: Localized.Social.opensea
        case .instagram: Localized.Social.instagram
        case .magicEden: Localized.Social.magiceden
        case .tikTok: Localized.Social.tiktok
        }
    }
}

extension PerpetualMarginType {
    public var title: String {
        switch self {
        case .cross: Localized.Perpetual.Margin.cross
        case .isolated: Localized.Perpetual.Margin.isolated
        }
    }
}

extension Resource {
    public var title: String {
        switch self {
        case .bandwidth: Localized.Stake.Resource.bandwidth
        case .energy: Localized.Stake.Resource.energy
        }
    }
}

extension SimulationPayloadFieldKind {
    public var title: String? {
        switch self {
        case .contract: Localized.Asset.contract
        case .method: Localized.Common.method
        case .token: Localized.Common.token
        case .spender: Localized.Transfer.to
        case .value: Localized.Perpetual.value
        case .expiration: Localized.Common.expiration
        case .custom: nil
        }
    }
}

extension TransactionState {
    public var statusTitle: String {
        switch self {
        case .confirmed: Localized.Transaction.Status.confirmed
        case .pending, .inTransit: Localized.Transaction.Status.pending
        case .failed: Localized.Transaction.Status.failed
        case .reverted: Localized.Transaction.Status.reverted
        case .refunded: Localized.Transaction.Status.refunded
        }
    }
}

extension GemTransactionStateTone {
    public var infoDescription: String {
        switch self {
        case .pending: Localized.Info.Transaction.Pending.description
        case .success: Localized.Info.Transaction.Success.description
        case .error, .refunded: Localized.Info.Transaction.Error.description
        }
    }
}

extension Primitives.PerpetualDirection {
    public var title: String {
        switch self {
        case .short: Localized.Perpetual.short
        case .long: Localized.Perpetual.long
        }
    }
}

extension Primitives.FeePriority {
    public var title: String {
        switch self {
        case .normal: Localized.FeeRates.normal
        case .fast: Localized.FeeRates.fast
        }
    }
}

extension GemApprovalValue {
    public func title(symbol: String, formatter: ValueFormatter, decimals: Int) -> String {
        switch self {
        case .unlimited: Localized.Simulation.Header.unlimitedAsset(symbol)
        case let .exact(value): formatter.string(BigInt(value), decimals: decimals, currency: symbol)
        }
    }
}

extension ConnectionStatus {
    public var bannerTitle: String? {
        switch self {
        case .online: nil
        case .noInternet: Localized.Errors.noInternetConnection
        case .noService: Localized.Errors.noServiceConnection
        }
    }
}

extension FeeUnitType {
    public func suffix(symbol: String) -> String {
        switch self {
        case .satVb: Localized.FeeRate.satvB
        case .gwei: Localized.FeeRate.gwei
        case .native: symbol
        }
    }
}

extension GemAssetMenuAction {
    public var title: String? {
        switch self {
        case .addToWallet: Localized.Asset.addToWallet
        case .copyAddress: Localized.Wallet.copyAddress
        case .pin, .hide: nil
        }
    }
}

extension PerpetualType {
    public var confirmedTitle: String {
        switch self {
        case let .open(data): Localized.Perpetual.openDirection(data.direction.toPrimitives().title)
        case .close: Localized.Perpetual.closePosition
        case .modify: Localized.Perpetual.modifyPosition
        case .increase: Localized.Perpetual.increasePosition
        case .reduce: Localized.Perpetual.reducePosition
        }
    }
}

extension ScanReceiveMode {
    public var title: String {
        switch self {
        case .scan: Localized.Wallet.scan
        case .receive: Localized.Wallet.receive
        }
    }
}

extension GemTransactionRowSubtitle {
    public var prefix: String? {
        switch self {
        case .toAddress, .toResource: Localized.Transfer.to
        case .fromAddress, .fromResource: Localized.Transfer.from
        case .price: Localized.Asset.price
        case .none: nil
        }
    }
}

extension VerificationStatus {
    public var statusTitle: String {
        switch self {
        case .verified: ""
        case .unverified: Localized.Asset.Verification.unverified
        case .suspicious: Localized.Asset.Verification.suspicious
        }
    }

    public var statusDescription: String {
        switch self {
        case .verified: ""
        case .unverified: Localized.Info.AssetStatus.Unverified.description
        case .suspicious: Localized.Info.AssetStatus.Suspicious.description
        }
    }
}

extension FiatQuoteType {
    public func title(asset: String) -> String {
        switch self {
        case .buy: Localized.Buy.title(asset)
        case .sell: Localized.Sell.title(asset)
        }
    }

    public var action: String {
        switch self {
        case .buy: Localized.Wallet.buy
        case .sell: Localized.Wallet.sell
        }
    }
}

extension GemEmptyStateText {
    public func text(symbol: String) -> String {
        switch self {
        case .nftsTitle: Localized.Nft.State.Empty.title
        case .nftsDescription: Localized.Nft.State.Empty.description
        case .priceAlertsTitle: Localized.PriceAlerts.State.Empty.title
        case .priceAlertsDescription: Localized.PriceAlerts.State.Empty.description
        case .contactsTitle: Localized.Contacts.State.Empty.title
        case .contactsDescription: Localized.Contacts.State.Empty.description
        case .assetTitle: Localized.Asset.State.Empty.title
        case .assetDescription: Localized.Asset.State.Empty.description(symbol)
        case .activityTitle: Localized.Activity.State.Empty.title
        case .activityDescription: Localized.Activity.State.Empty.description
        case .stakeTitle: Localized.Stake.State.Empty.title
        case .stakeDescription: Localized.Stake.State.Empty.description(symbol)
        case .earnTitle: Localized.Earn.State.Empty.title
        case .earnDescription: Localized.Earn.State.Empty.description(symbol)
        case .walletConnectTitle: Localized.WalletConnect.noActiveConnections
        case .walletConnectDescription: Localized.WalletConnect.State.Empty.description
        case .recentsTitle: Localized.RecentActivity.State.Empty.title
        case .recentsDescription: Localized.RecentActivity.State.Empty.description
        case .notificationsTitle: Localized.Notifications.Inapp.State.Empty.title
        case .notificationsDescription: Localized.Notifications.Inapp.State.Empty.description
        case .watchWalletTitle: Localized.Wallet.watchEmptyStateTitle
        case .watchWalletDescription: Localized.Info.WatchWallet.description
        case .noAssetsFoundTitle: Localized.Assets.noAssetsFound
        case .searchDescription: Localized.Search.State.Empty.description
        case .searchAssetsDescription: Localized.Assets.State.Empty.searchDescription
        case .searchActivityTitle: Localized.Activity.State.Empty.searchTitle
        case .searchActivityDescription: Localized.Activity.State.Empty.searchDescription
        case .searchNetworksTitle: Localized.Networks.State.Empty.searchTitle
        case .searchPerpetualsTitle: Localized.Perpetuals.EmptyState.noMarketsFound
        }
    }
}

extension GemEmptyStateAction {
    public var title: String {
        switch self {
        case .buy: Localized.Wallet.buy
        case .swap: Localized.Wallet.swap
        case .receive: Localized.Wallet.receive
        case .addCustomToken: Localized.Assets.addCustomToken
        case .manageTokenList: Localized.Wallet.manageTokenList
        case .clearFilters: Localized.Filter.clear
        }
    }
}

extension GemErrorText {
    public var text: String {
        switch self {
        case .cancelled: Localized.Errors.cancelled
        case .networkOffline: Localized.Errors.networkOffline
        case let .networkMessage(text): Localized.Errors.networkError(text)
        case let .networkStatus(status): Localized.Errors.networkError(status)
        case .invalidNetworkId: Localized.Errors.invalidNetworkId
        case .invalidUrl: Localized.Errors.invalidUrl
        case .notSupported: Localized.Errors.notSupported
        case .unsupportedChain: Localized.Errors.Connections.unsupportedChain
        case .maliciousOrigin: Localized.Errors.Connections.maliciousOrigin
        case .noSupportedWallets: Localized.Errors.Connections.noSupportedWallets
        case let .payment(status): status.errorText
        case let .message(text): text
        }
    }
}

extension PaymentStatus {
    public var errorText: String {
        switch self {
        case .requiresAction, .failed: Localized.Errors.paymentFailed
        case .processing: Localized.Errors.paymentInProgress
        case .succeeded: Localized.Errors.paymentPaid
        case .expired: Localized.Errors.paymentExpired
        case .cancelled: Localized.Errors.paymentCancelled
        }
    }
}

extension GemSelectAssetTitle {
    public var text: String {
        switch self {
        case .send: Localized.Wallet.send
        case .payWith: Localized.Transfer.payWith
        case .receive: Localized.Wallet.receive
        case .receiveCollection: Localized.Wallet.receiveCollection
        case .buy: Localized.Wallet.buy
        case .swapPay: Localized.Swap.youPay
        case .swapReceive: Localized.Swap.youReceive
        case .manageTokenList: Localized.Wallet.manageTokenList
        case .selectAsset: Localized.Assets.selectAsset
        case .deposit: Localized.Wallet.deposit
        case .withdraw: Localized.Wallet.withdraw
        case .search: Localized.Assets.selectAsset
        }
    }
}

extension GemSelectAssetSection {
    public var text: String {
        switch self {
        case .assets: Localized.Assets.title
        case .networks: Localized.Settings.Networks.title
        }
    }
}

extension GemDayLabel {
    var title: String? {
        switch self {
        case .today: Localized.Date.today
        case .yesterday: Localized.Date.yesterday
        case .date: nil
        }
    }
}

public extension GemContactAddressField {
    var title: String {
        switch self {
        case .network: Localized.Transfer.network
        case .address: Localized.Common.address
        case .memo: Localized.Transfer.memo
        }
    }
}

public extension GemBannerTitle {
    var text: String {
        switch self {
        case let .stake(assetName): Localized.Banner.Stake.title(assetName)
        case .accountActivation: Localized.Banner.AccountActivation.title
        case .warning: Localized.Common.warning
        case .activateAsset: Localized.Transfer.ActivateAsset.title
        case .suspiciousAsset: Localized.Banner.AssetStatus.title
        case .onboarding: Localized.Banner.Onboarding.title
        case .tradePerpetuals: Localized.Banner.Perpetuals.title
        }
    }
}

public extension GemBannerDescription {
    func text(amount: (GemBannerAmount) -> String) -> String {
        switch self {
        case let .stake(assetSymbol): Localized.Banner.Stake.description(assetSymbol)
        case let .accountActivation(networkName, fee): Localized.Banner.AccountActivation.description(networkName, amount(fee))
        case let .multiSignatureBlocked(networkName): Localized.Warnings.multiSignatureBlocked(networkName)
        case let .activateAsset(assetSymbol, networkName): Localized.Banner.ActivateAsset.description(assetSymbol, networkName)
        case .suspiciousAsset: Localized.Banner.AssetStatus.description
        case .onboarding: Localized.Banner.Onboarding.description
        case .tradePerpetuals: Localized.Banner.Perpetuals.description
        }
    }
}

extension GemRecipientErrorDisplay: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case let .invalidAddress(network): Localized.Errors.invalidAssetAddress(network.boldMarkdown())
        }
    }
}
