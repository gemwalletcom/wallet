// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import enum Gemstone.AddressType
import enum Gemstone.DelegationState
import enum Gemstone.FeeOption
import enum Gemstone.GemApprovalValue
import enum Gemstone.GemAssetMenuAction
import enum Gemstone.GemBalanceRowValue
import enum Gemstone.GemBannerDescription
import enum Gemstone.GemBannerTitle
import enum Gemstone.GemContactAddressField
import enum Gemstone.GemCopyKind
import enum Gemstone.GemDayLabel
import enum Gemstone.GemEmptyStateAction
import enum Gemstone.GemEmptyStateText
import enum Gemstone.GemErrorText
import enum Gemstone.GemFiatTransactionBadge
import enum Gemstone.GemHeaderButtonKind
import enum Gemstone.GemListRowTitle
import enum Gemstone.GemListSectionFooter
import enum Gemstone.GemListSectionTitle
import enum Gemstone.GemLocalizedText
import enum Gemstone.GemPriceAlertLabel
import struct Gemstone.GemPriceAlertRow
import enum Gemstone.GemPriceAlertText
import enum Gemstone.GemRecipientError
import enum Gemstone.GemRecipientErrorDisplay
import enum Gemstone.GemSelectAssetSection
import enum Gemstone.GemSelectAssetTitle
import enum Gemstone.GemSimulationPayloadTitle
import enum Gemstone.GemTransactionRowSubtitle
import enum Gemstone.GemTransactionStateTone
import enum Gemstone.GemTransactionTitle
import enum Gemstone.GemTriggerOrder
import enum Gemstone.GemWalletSubtitle
import enum Gemstone.LinkType
import enum Gemstone.PaymentStatus
import enum Gemstone.PerpetualDirection
import class Gemstone.PriceChangeCalculator
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public extension FeeOption {
    var title: String {
        switch self {
        case .tokenAccountCreation: Localized.Banner.AccountActivation.title
        }
    }
}

public extension GemLocalizedText {
    var text: String {
        switch self {
        case let .walletDefaultName(index):
            Localized.Wallet.defaultName(Int(index))
        case let .walletDefaultNameChain(networkName, index):
            Localized.Wallet.defaultNameChain(networkName, Int(index))
        case .walletMulticoin:
            Localized.Wallet.multicoin
        case let .chainNetworkName(chain):
            Chain(core: chain).networkName
        case let .delegationState(state):
            state.title
        case let .transactionState(state):
            state.toPrimitives().statusTitle
        case let .resource(resource):
            resource.toPrimitives().title
        case let .feeRate(rate, unit):
            switch unit {
            case .satVb: "\(rate.text()) \(Localized.FeeRate.satvB)"
            case .gwei: "\(rate.text()) \(Localized.FeeRate.gwei)"
            case .native: rate.text()
            }
        case let .text(text):
            text
        case let .number(number):
            number.text()
        case .none:
            Localized.Common.none
        case .slippageAuto:
            Localized.Swap.slippageAuto
        case .rewardsUnverified:
            Localized.Rewards.Unverified.description
        case let .rewardsPending(countdown):
            CountdownFormatter().string(parts: countdown).map { Localized.Rewards.Pending.description($0) } ?? .empty
        case .rewardsPendingReady:
            Localized.Rewards.Pending.descriptionReady
        case .errorOccurred:
            Localized.Errors.errorOccurred
        case .unlimitedApprovalWarning:
            Localized.Simulation.Warning.UnlimitedTokenApproval.description
        case .externallyOwnedSpenderWarning:
            Localized.Simulation.warningExternallyOwnedSpenderDescription
        case .suspiciousAddressDescription:
            Localized.Common.suspiciousAddressDescription
        case let .addressType(addressType):
            addressType.title
        case .invalidTokenId:
            Localized.Errors.Token.invalidId
        case let .triggerOrder(order, price):
            "\(order.title): \(price?.text() ?? Placeholder.empty)"
        case let .pnl(amount, percent):
            PriceChangeCalculator().pnlText(formattedAmount: amount.text(), formattedPercentage: percent.text())
        case let .margin(amount, marginType):
            "\(amount.text()) (\(marginType.toPrimitives().title))"
        case let .position(direction, leverage):
            "\(direction.toPrimitives().title.uppercased()) \(leverage.text())"
        case let .apr(value):
            Localized.Stake.apr(value?.text() ?? .empty)
        case let .priceImpactWarning(percent, symbol):
            Localized.Swap.PriceImpactWarning.description(percent.text(), symbol)
        case let .balance(amount):
            Localized.Transfer.balance(amount.text())
        case .nftCollections:
            Localized.Nft.collections
        case .nftUnverified:
            Localized.Asset.Verification.unverified
        case let .rewardsRedeemAsset(value):
            Localized.Rewards.WaysSpend.Asset.title(value.text())
        case let .signInWith(chain):
            Localized.Common.signInWith(Chain(core: chain).networkName)
        case .reviewRequest:
            Localized.Transfer.reviewRequest
        case .enableDeveloper:
            Localized.Settings.enableValue(Localized.Settings.developer)
        case .disableDeveloper:
            Localized.Settings.disableValue(Localized.Settings.developer)
        case let .stakeProvider(provider):
            switch provider {
            case .stake: Localized.Transfer.Stake.title
            case .earn: Localized.Common.earn
            }
        case let .positionChange(change, direction):
            switch change {
            case .increase: Localized.Perpetual.increaseDirection(direction.toPrimitives().title)
            case .reduce: Localized.Perpetual.reduceDirection(direction.toPrimitives().title)
            }
        case let .perpetualConfirmed(action):
            switch action {
            case let .open(direction): Localized.Perpetual.openDirection(direction.toPrimitives().title)
            case .close: Localized.Perpetual.closePosition
            case .modify: Localized.Perpetual.modifyPosition
            case .increase: Localized.Perpetual.increasePosition
            case .reduce: Localized.Perpetual.reducePosition
            }
        }
    }
}

public extension GemBalanceRowValue {
    var text: String {
        switch self {
        case let .amount(amount): amount.text()
        case let .apr(apr): Localized.Stake.apr(apr?.text() ?? "")
        }
    }
}

public extension Gemstone.DelegationState {
    var title: String {
        switch self {
        case .active: Localized.Stake.active
        case .pending: Localized.Stake.pending
        case .inactive: Localized.Stake.inactive
        case .activating: Localized.Stake.activating
        case .deactivating: Localized.Stake.deactivating
        case .awaitingWithdrawal: Localized.Stake.awaitingWithdrawal
        }
    }
}

public extension GemPriceAlertText {
    var text: String {
        switch self {
        case .empty: Placeholder.empty
        case let .number(value): value.text()
        case let .label(label): label.text
        }
    }
}

public extension GemPriceAlertRow {
    var prefixText: String {
        prefix.text
    }

    var suffixText: String {
        suffix.text
    }
}

public extension GemPriceAlertLabel {
    var text: String {
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

public extension GemWalletSubtitle {
    var text: String {
        switch self {
        case .multicoin: Localized.Wallet.multicoin
        case let .address(value): value
        }
    }
}

public extension GemFiatTransactionBadge {
    var text: String {
        switch self {
        case .pending: Localized.Transaction.Status.pending
        case .failed: Localized.Transaction.Status.failed
        }
    }
}

public extension GemHeaderButtonKind {
    var title: String {
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

public extension ChartPeriod {
    var title: String {
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

public extension LinkType {
    var title: String {
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

public extension PerpetualMarginType {
    var title: String {
        switch self {
        case .cross: Localized.Perpetual.Margin.cross
        case .isolated: Localized.Perpetual.Margin.isolated
        }
    }
}

public extension Resource {
    var title: String {
        switch self {
        case .bandwidth: Localized.Stake.Resource.bandwidth
        case .energy: Localized.Stake.Resource.energy
        }
    }
}

public extension GemSimulationPayloadTitle {
    var text: String {
        switch self {
        case .contract: Localized.Asset.contract
        case .method: Localized.Common.method
        case .token: Localized.Common.token
        case .spender: Localized.Transfer.to
        case .value: Localized.Perpetual.value
        case .expiration: Localized.Common.expiration
        case let .custom(label): label
        }
    }
}

public extension TransactionState {
    var statusTitle: String {
        switch self {
        case .confirmed: Localized.Transaction.Status.confirmed
        case .pending, .inTransit: Localized.Transaction.Status.pending
        case .failed: Localized.Transaction.Status.failed
        case .reverted: Localized.Transaction.Status.reverted
        case .refunded: Localized.Transaction.Status.refunded
        }
    }
}

public extension GemTransactionStateTone {
    var infoDescription: String {
        switch self {
        case .pending: Localized.Info.Transaction.Pending.description
        case .success: Localized.Info.Transaction.Success.description
        case .error, .refunded: Localized.Info.Transaction.Error.description
        }
    }
}

public extension Primitives.PerpetualDirection {
    var title: String {
        switch self {
        case .short: Localized.Perpetual.short
        case .long: Localized.Perpetual.long
        }
    }

    var increaseTitle: String {
        Localized.Perpetual.increaseDirection(title)
    }

    var reduceTitle: String {
        Localized.Perpetual.reduceDirection(title)
    }
}

public extension Primitives.FeePriority {
    var title: String {
        switch self {
        case .normal: Localized.FeeRates.normal
        case .fast: Localized.FeeRates.fast
        }
    }
}

public extension GemApprovalValue {
    func title(symbol: String, formatter: ValueFormatter, decimals: Int) -> String {
        switch self {
        case .unlimited: Localized.Simulation.Header.unlimitedAsset(symbol)
        case let .exact(value): formatter.string(BigInt(value), decimals: decimals, currency: symbol)
        }
    }
}

public extension ConnectionStatus {
    var bannerTitle: String? {
        switch self {
        case .online: nil
        case .noInternet: Localized.Errors.noInternetConnection
        case .noService: Localized.Errors.noServiceConnection
        }
    }
}

public extension FeeUnitType {
    func suffix(symbol: String) -> String {
        switch self {
        case .satVb: Localized.FeeRate.satvB
        case .gwei: Localized.FeeRate.gwei
        case .native: symbol
        }
    }
}

public extension GemAssetMenuAction {
    var title: String? {
        switch self {
        case .addToWallet: Localized.Asset.addToWallet
        case .copyAddress: Localized.Wallet.copyAddress
        case .pin, .hide: nil
        }
    }
}

public extension ScanReceiveMode {
    var title: String {
        switch self {
        case .scan: Localized.Wallet.scan
        case .receive: Localized.Wallet.receive
        }
    }
}

public extension GemTransactionRowSubtitle {
    var prefix: String? {
        switch self {
        case .toAddress, .toResource: Localized.Transfer.to
        case .fromAddress, .fromResource: Localized.Transfer.from
        case .price: Localized.Asset.price
        case .none: nil
        }
    }
}

public extension VerificationStatus {
    var statusTitle: String {
        switch self {
        case .verified: ""
        case .unverified: Localized.Asset.Verification.unverified
        case .suspicious: Localized.Asset.Verification.suspicious
        }
    }

    var statusDescription: String {
        switch self {
        case .verified: ""
        case .unverified: Localized.Info.AssetStatus.Unverified.description
        case .suspicious: Localized.Info.AssetStatus.Suspicious.description
        }
    }
}

public extension FiatQuoteType {
    func title(asset: String) -> String {
        switch self {
        case .buy: Localized.Buy.title(asset)
        case .sell: Localized.Sell.title(asset)
        }
    }

    var action: String {
        switch self {
        case .buy: Localized.Wallet.buy
        case .sell: Localized.Wallet.sell
        }
    }
}

public extension GemEmptyStateText {
    func text(symbol: String) -> String {
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
        case .validatorsTitle: Localized.Stake.State.Empty.validatorsTitle
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

public extension GemEmptyStateAction {
    var title: String {
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

public extension GemErrorText {
    var text: String {
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
        case .invalidSecretPhrase: Localized.Errors.Import.invalidSecretPhrase
        case let .invalidSecretPhraseWords(words): Localized.Errors.Import.invalidSecretPhraseWord(words.joined(separator: ", "))
        case .invalidPrivateKey: Localized.Errors.Import.invalidPrivateKey
        case .invalidAddress: Localized.Errors.invalidAddressName
        case .noAccountForChain: Localized.Errors.walletAccountMissing
        case .unknown: Localized.Errors.unknown
        case let .message(text): text
        }
    }
}

extension PaymentStatus {
    public var errorText: String {
        Localized.Errors.paymentStatus(text)
    }

    var text: String {
        switch self {
        case .requiresAction, .failed: Localized.Transaction.Status.failed
        case .processing: Localized.Transaction.Status.inprogress
        case .succeeded: Localized.Transaction.Status.completed
        case .expired: Localized.Transaction.Status.expired
        case .cancelled: Localized.Errors.cancelled
        }
    }
}

public extension GemSelectAssetTitle {
    var text: String {
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

public extension GemSelectAssetSection {
    var text: String {
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
    var text: String {
        switch self {
        case let .stake(assetSymbol): Localized.Banner.Stake.description(assetSymbol)
        case let .accountActivation(networkName, fee): Localized.Banner.AccountActivation.description(networkName, fee.text())
        case let .externallyControlledAccount(networkName): Localized.Warnings.externallyControlledAccount(networkName)
        case let .activateAsset(assetSymbol, networkName): Localized.Banner.ActivateAsset.description(assetSymbol, networkName)
        case .suspiciousAsset: Localized.Banner.AssetStatus.description
        case .onboarding: Localized.Banner.Onboarding.description
        case .tradePerpetuals: Localized.Banner.Perpetuals.description
        }
    }
}

extension GemErrorText: @retroactive LocalizedError {
    public var errorDescription: String? {
        text
    }
}

extension GemRecipientError: @retroactive LocalizedError {
    public var errorDescription: String? { display().errorDescription }
}

extension GemRecipientErrorDisplay: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case let .invalidAddress(network): Localized.Errors.invalidAssetAddress(network.boldMarkdown())
        }
    }
}

public extension GemListSectionTitle {
    var text: String? {
        switch self {
        case .none: nil
        case .balances: Localized.Asset.balances
        case .info: Localized.Common.info
        case .community: Localized.Settings.community
        case .manage: Localized.Common.manage
        case .resources: Localized.Asset.resources
        case .socialLinks: Localized.Social.links
        case .properties: Localized.Nft.properties
        }
    }
}

public extension GemListSectionFooter {
    var text: String? {
        switch self {
        case .none: nil
        case .authentication: Localized.Lock.footer
        }
    }
}

public extension GemListRowTitle {
    var text: String {
        switch self {
        case .api: "API"
        case .stream: Localized.Nodes.stream
        case .gemWalletNode: Localized.Nodes.gemWalletNode
        case .name: Localized.Asset.name
        case .network: Localized.Transfer.network
        case .address: Localized.Common.address
        case .available: Localized.Asset.Balances.available
        case .stake: Localized.Wallet.stake
        case .freeze: Localized.Transfer.Freeze.title
        case .unfreeze: Localized.Transfer.Unfreeze.title
        case .claimRewards: Localized.Transfer.ClaimRewards.title
        case .earn: Localized.Common.earn
        case .pendingUnconfirmed: Localized.Stake.pending
        case .reserved: Localized.Asset.Balances.reserved
        case .error: Localized.Errors.errorOccurred
        case .termsOfService: Localized.Settings.termsOfServices
        case .privacyPolicy: Localized.Settings.privacyPolicy
        case .website: Localized.Settings.website
        case .version: Localized.Settings.version
        case .updateApp: Localized.UpdateApp.title
        case .wallets: Localized.Wallets.title
        case .security: Localized.Settings.security
        case .notifications: Localized.Settings.Notifications.title
        case .preferences: Localized.Settings.Preferences.title
        case .walletConnect: Localized.WalletConnect.title
        case .support: Localized.Settings.support
        case .rewards: Localized.Rewards.title
        case .myReferralCode: Localized.Rewards.myReferralCode
        case .referrals: Localized.Rewards.referrals
        case .points: Localized.Rewards.points
        case .invitedBy: Localized.Rewards.invitedBy
        case .aboutUs: Localized.Settings.aboutus
        case .developer: Localized.Settings.developer
        case .authentication: Localized.Settings.enablePasscode
        case .lockPeriod: Localized.Lock.requireAuthentication
        case .privacyLock: Localized.Lock.privacyLock
        case .hideBalance: Localized.Settings.hideBalance
        case .currency: Localized.Settings.currency
        case .language: Localized.Settings.language
        case .appearance: Localized.Settings.appearanceTitle
        case .networks: Localized.Settings.Networks.title
        case .contacts: Localized.Contacts.title
        case .perpetuals: Localized.Perpetuals.title
        case .perpetualLeverage: Localized.Settings.Preferences.Perpetual.defaultLeverage
        case .perpetualTakeProfit: Localized.Settings.Preferences.Perpetual.defaultTakeProfit
        case .perpetualStopLoss: Localized.Settings.Preferences.Perpetual.defaultStopLoss
        case .dailyVolume: Localized.Markets.dailyVolume
        case .openInterest: Localized.Info.Perpetual.OpenInterest.title
        case .fundingApr: Localized.Info.Perpetual.FundingApr.title
        case .stakeApr: Localized.Stake.apr("")
        case .lockTime: Localized.Stake.lockTime
        case .minimumAmount: Localized.Stake.minimumAmount
        case .networkFee: Localized.Transfer.networkFee
        case .validator: Localized.Stake.validator
        case .provider: Localized.Common.provider
        case .status: Localized.Transaction.status
        case .activeIn: Localized.Stake.activeIn
        case .availableIn: Localized.Stake.availableIn
        case .date: Localized.Transaction.date
        case .resource: Localized.Stake.resource
        case .price: Localized.Asset.price
        case .pnl: Localized.Perpetual.pnl
        case .pin: Localized.Common.pin
        case .unpin: Localized.Common.unpin
        case .addToWallet: Localized.Asset.addToWallet
        case .priceAlerts: Localized.Settings.PriceAlerts.title
        case .setPriceAlert: Localized.PriceAlerts.SetAlert.title
        case .energy: Localized.Stake.Resource.energy
        case .bandwidth: Localized.Stake.Resource.bandwidth
        case .rewardsUnverified: Localized.Rewards.Unverified.title
        case .rewardsPending: Localized.Rewards.Pending.title
        case .warning: Localized.Common.warning
        case .suspiciousAddress: Localized.Common.suspiciousAddress
        case .unlimitedApproval: Localized.Simulation.Warning.UnlimitedTokenApproval.title
        case .nftCollectionApproval: Localized.Simulation.Warning.NftCollectionApproval.title
        case .symbol: Localized.Asset.symbol
        case .decimals: Localized.Asset.decimals
        case .type: Localized.Common.type
        case .autoClose: Localized.Perpetual.autoClose
        case .size: Localized.Perpetual.size
        case .entryPrice: Localized.Perpetual.entryPrice
        case .rate: Localized.Buy.rate
        case .liquidationPrice: Localized.Info.Perpetual.LiquidationPrice.title
        case .margin: Localized.Perpetual.margin
        case .position: Localized.Perpetual.position
        case .details: Localized.Common.details
        case .slippage: Localized.Swap.slippage
        case .priceImpact: Localized.Swap.priceImpact
        case .minimumReceive: Localized.Swap.minReceive
        case .estimatedTime: Localized.Swap.EstimatedTime.title
        case .estimatedConfirmation: Localized.Transaction.estimatedConfirmation
        case .marketPrice: Localized.Perpetual.marketPrice
        case .fundingPayments: Localized.Info.Perpetual.FundingPayments.title
        case .marketCap: Localized.Asset.marketCap
        case .fullyDilutedValuation: Localized.Info.FullyDilutedValuation.title
        case .tradingVolume: Localized.Asset.tradingVolume
        case .unrealizedPnl: Localized.Perpetual.unrealizedPnl
        case .accountLeverage: Localized.Perpetual.accountLeverage
        case .marginUsage: Localized.Perpetual.marginUsage
        case .allTimePnl: Localized.Perpetual.allTimePnl
        case .volume: Localized.Perpetual.volume
        case .circulatingSupply: Localized.Asset.circulatingSupply
        case .totalSupply: Localized.Asset.totalSupply
        case .maxSupply: Localized.Info.MaxSupply.title
        case .allTimeHigh: Localized.Asset.allTimeHigh
        case .allTimeLow: Localized.Asset.allTimeLow
        case .wallet: Localized.Common.wallet
        case .contract: Localized.Asset.contract
        case .tokenId: Localized.Asset.tokenId
        case .collection: Localized.Nft.collection
        }
    }
}

extension GemCopyKind {
    func copiedMessage(display: String) -> String {
        switch self {
        case .plain: Localized.Common.copied(display)
        case .secretPhrase: Localized.Common.copied(Localized.Common.secretPhrase)
        case .privateKey: Localized.Common.copied(Localized.Common.privateKey)
        case let .address(chain): Localized.Common.copied(String(format: "%@ (%@)", Chain(core: chain).networkName, display))
        }
    }
}

extension Gemstone.AddressType {
    var title: String {
        switch self {
        case .address: Localized.Common.address
        case .contract: Localized.Asset.contract
        case .asset: Localized.Common.token
        case .validator: Localized.Stake.validator
        case .contact: Localized.Contacts.contact
        case .internalWallet: Localized.Common.wallet
        }
    }
}

extension GemTriggerOrder {
    var title: String {
        switch self {
        case .takeProfit: Localized.Perpetual.takeProfit
        case .stopLoss: Localized.Perpetual.stopLoss
        }
    }
}
