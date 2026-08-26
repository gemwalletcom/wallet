// Copyright (c). Gem Wallet. All rights reserved.

import Components
import protocol Gemstone.GemAssetsServiceProtocol
import protocol Gemstone.GemBalanceServiceProtocol
import protocol Gemstone.GemBannerServiceProtocol
import protocol Gemstone.GemTransactionsServiceProtocol
import GemstoneServices
import protocol Gemstone.GemExplorerServiceProtocol
import GemstonePrimitives
import Localization
import Preferences
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI
import UIKit

@Observable
@MainActor
public final class AssetSceneViewModel: Sendable {
    private let assetsEnabler: any AssetsEnabler
    private let balanceService: any GemBalanceServiceProtocol
    private let assetsService: any GemAssetsServiceProtocol
    private let transactionsService: any GemTransactionsServiceProtocol
    private let priceUpdater: any PriceUpdater
    private let bannerService: any GemBannerServiceProtocol

    private let preferences: ObservablePreferences = .default

    let explorerService: any GemExplorerServiceProtocol
    public let priceAlertService: PriceAlertService

    private var isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>

    public var isPresentingToastMessage: ToastMessage?
    public var isPresentingAssetSheet: AssetSheetType?

    public var input: AssetSceneInput
    public let assetQuery: ObservableQuery<ChainAssetRequest>
    public let bannersQuery: ObservableQuery<BannersRequest>
    public let transactionsQuery: ObservableQuery<TransactionsRequest>

    public init(
        assetsEnabler: any AssetsEnabler,
        balanceService: any GemBalanceServiceProtocol,
        assetsService: any GemAssetsServiceProtocol,
        transactionsService: any GemTransactionsServiceProtocol,
        priceUpdater: any PriceUpdater,
        priceAlertService: PriceAlertService,
        bannerService: any GemBannerServiceProtocol,
        explorerService: any GemExplorerServiceProtocol,
        input: AssetSceneInput,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
    ) {
        self.assetsEnabler = assetsEnabler
        self.balanceService = balanceService
        self.assetsService = assetsService
        self.transactionsService = transactionsService
        self.priceUpdater = priceUpdater
        self.priceAlertService = priceAlertService
        self.bannerService = bannerService
        self.explorerService = explorerService

        self.input = input
        assetQuery = ObservableQuery(
            input.assetRequest,
            initialValue: ChainAssetData(
                assetData: AssetData.with(asset: input.asset),
                feeAssetData: AssetData.with(asset: input.asset.chain.asset),
            ),
        )
        bannersQuery = ObservableQuery(input.bannersRequest, initialValue: [])
        transactionsQuery = ObservableQuery(input.transactionsRequest, initialValue: [])
        self.isPresentingSelectedAssetInput = isPresentingSelectedAssetInput
    }

    public var chainAssetData: ChainAssetData {
        assetQuery.value
    }

    public var banners: [Banner] {
        bannersQuery.value
    }

    public var transactions: [TransactionExtended] {
        transactionsQuery.value
    }

    public var assetData: AssetData {
        chainAssetData.assetData
    }

    private var asset: Asset {
        assetData.asset
    }

    private var wallet: Wallet {
        walletModel.wallet
    }

    public var title: String {
        assetModel.name
    }

    var balancesTitle: String {
        Localized.Asset.balances
    }

    var networkField: ListItemField {
        ListItemField(title: Localized.Transfer.network, value: assetModel.networkFullName)
    }

    var resourcesTitle: String {
        Localized.Asset.resources
    }

    var energyField: ListItemField {
        ListItemField(title: ResourceViewModel(resource: .energy).title, value: feeAssetDataModel.energyText)
    }

    var bandwidthField: ListItemField {
        ListItemField(title: ResourceViewModel(resource: .bandwidth).title, value: feeAssetDataModel.bandwidthText)
    }

    var networkDestination: AssetNetworkDestination? {
        switch asset.id.type {
        case .native:
            break
        case .token:
            if asset.chain.hasNativeAsset {
                return .asset(asset.chain.asset)
            }
        }
        if AssetConfiguration.supportedChainsWithTokens.contains(asset.chain) {
            return .assets(asset.chain)
        }
        return nil
    }

    var showBalances: Bool {
        assetDataModel.showBalances || showProviderBalance(for: .earn)
    }

    var showReservedBalance: Bool {
        assetDataModel.hasReservedBalance
    }

    var showPendingUnconfirmedBalance: Bool {
        assetDataModel.hasPendingUnconfirmedBalance
    }

    var showResources: Bool {
        assetDataModel.showResources
    }

    var showTransactions: Bool {
        transactions.isNotEmpty
    }

    var showManageToken: Bool {
        !assetData.metadata.isBalanceEnabled
    }

    var canSign: Bool {
        wallet.canSign
    }

    var pinText: String {
        assetData.metadata.isPinned ? Localized.Common.unpin : Localized.Common.pin
    }

    var pinSystemImage: String {
        assetData.metadata.isPinned ? SystemImage.unpin : SystemImage.pin
    }

    var pinImage: Image {
        Image(systemName: pinSystemImage)
    }

    var enableText: String {
        assetData.metadata.isBalanceEnabled ? Localized.Asset.hideFromWallet : Localized.Asset.addToWallet
    }

    var enableImage: Image {
        Image(systemName: enableSystemImage)
    }

    var enableSystemImage: String {
        assetData.metadata.isBalanceEnabled ? SystemImage.minusCircle : SystemImage.plusCircle
    }

    var reservedBalanceUrl: URL? {
        assetModel.asset.chain.accountActivationFeeUrl
    }

    var showEarnButton: Bool {
        #if DEBUG
            assetData.metadata.isEarnEnabled && !wallet.isViewOnly && !showProviderBalance(for: .earn)
        #else
            false
        #endif
    }

    var priceItemViewModel: PriceListItemViewModel {
        PriceListItemViewModel(
            title: Localized.Asset.price,
            model: assetDataModel.priceViewModel,
        )
    }

    var networkAssetImage: AssetImage {
        AssetIdViewModel(assetId: assetModel.asset.chain.assetId).networkAssetImage
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        let buy = assetData.metadata.isBuyEnabled ? onSelectBuy : nil
        let swap = buy == nil && assetData.metadata.isSwapEnabled ? onSelectSwap : nil
        return EmptyContentTypeViewModel(
            type: .asset(symbol: assetModel.symbol, buy: buy, swap: swap, isViewOnly: wallet.isViewOnly),
        )
    }

    var assetDataModel: AssetDataViewModel {
        AssetDataViewModel(
            assetData: assetData,
            formatter: .auto,
            currencyCode: preferences.preferences.currency,
        )
    }

    var assetBannerViewModel: AssetSceneBannersViewModel {
        AssetSceneBannersViewModel(wallet: wallet, assetData: assetData, banners: banners)
    }

    var assetHeaderModel: AssetHeaderViewModel {
        AssetHeaderViewModel(
            assetDataModel: assetDataModel,
            walletModel: walletModel,
            bannerEventsViewModel: HeaderBannerEventViewModel(events: assetBannerViewModel.allBanners.map(\.event)),
        )
    }

    public var shareAssetUrl: URL {
        DeepLink.asset(assetDataModel.asset.id).url
    }

    public var assetModel: AssetViewModel {
        AssetViewModel(asset: assetData.asset)
    }

    public var walletModel: WalletViewModel {
        WalletViewModel(wallet: input.wallet)
    }

    public var optionsImage: Image {
        Images.System.ellipsis
    }

    public var priceAlertsSystemImage: String {
        assetData.isPriceAlertsEnabled ? SystemImage.bellFill : SystemImage.bell
    }

    public var priceAlertsImage: Image {
        Image(systemName: priceAlertsSystemImage)
    }

    public var showPriceAlerts: Bool {
        priceAlertsViewModel.hasPriceAlerts && assetDataModel.isPriceAvailable
    }

    public var menuItems: [ActionMenuItemType] {
        [.button(title: viewAddressOnTitle, systemImage: SystemImage.globe, action: { self.onSelect(url: self.addressExplorerUrl) }),
         viewTokenOnTitle.map { .button(title: $0, systemImage: SystemImage.globe, action: { self.onSelect(url: self.tokenExplorerUrl) }) },
         .button(title: Localized.Common.share, systemImage: SystemImage.share, action: onSelectShareAsset)].compactMap(\.self)
    }

    var scoreViewModel: AssetScoreTypeViewModel {
        AssetScoreTypeViewModel(score: assetData.metadata.rankScore)
    }

    var showStatus: Bool {
        scoreViewModel.hasWarning
    }

    var priceAlertsViewModel: PriceAlertsViewModel {
        PriceAlertsViewModel(priceAlerts: assetData.priceAlerts)
    }

    var swapAssetType: SelectedAssetType {
        switch assetData.asset.id.type {
        case .native: .swap(assetData.asset, nil)
        case .token:
            if assetData.balance.available == .zero, assetData.asset.chain.hasNativeAsset {
                .swap(assetData.asset.chain.asset, assetData.asset)
            } else {
                .swap(assetData.asset, nil)
            }
        }
    }

    func showProviderBalance(for type: StakeProviderType) -> Bool {
        switch type {
        case .stake: assetDataModel.isStakeEnabled || assetData.balances.contains(where: { Self.showStakedBalanceTypes.contains($0.key) && $0.value > 0 })
        #if DEBUG
            case .earn: assetData.balance.earn > .zero
        #else
            case .earn: false
        #endif
        }
    }

    func balanceTitle(for type: StakeProviderType) -> String {
        switch type {
        case .stake: Localized.Wallet.stake
        case .earn: Localized.Common.earn
        }
    }

    func aprModel(for type: StakeProviderType) -> AprViewModel {
        AprViewModel(apr: assetDataModel.apr(for: type) ?? .zero)
    }
}

// MARK: - Business Logic

public extension AssetSceneViewModel {
    internal func fetchOnce() {
        Task {
            await fetch()
        }
        Task {
            await updateAsset()
        }
    }

    internal func fetch() async {
        await withTaskGroup(of: Void.self) { group in
            group.addTask { await self.updateWallet() }
            if assetData.priceAlerts.isNotEmpty {
                group.addTask { await self.updatePriceAlerts() }
            }
        }
    }

    internal func onSelectHeader(_ buttonType: HeaderButtonType) {
        let selectType: SelectedAssetType = switch buttonType {
        case .buy: .buy(assetData.asset, amount: nil)
        case .sell: .sell(assetData.asset, amount: nil)
        case .send: .send(.asset(assetData.asset))
        case .swap: swapAssetType
        case .receive: .receive(.asset)
        case .stake: .stake(assetData.asset)
        case .more, .deposit, .withdraw:
            fatalError()
        }
        isPresentingSelectedAssetInput.wrappedValue = SelectedAssetInput(
            type: selectType,
            assetData: assetData,
        )
    }

    internal func onSelectWalletHeaderInfo() {
        isPresentingAssetSheet = .info(.watchWallet)
    }

    internal func onSelectBanner(_ action: BannerAction) {
        switch action.type {
        case let .event(event):
            switch event {
            case .stake:
                onSelectHeader(.stake)
            case .activateAsset:
                isPresentingAssetSheet = .transfer(
                    TransferData(
                        type: .account(assetData.asset, .activate),
                        recipientData: RecipientData(
                            recipient: Recipient(
                                name: .none,
                                address: "",
                                memo: .none,
                            ),
                            amount: .none,
                        ),
                        amount: .exact(0),
                    ),
                )
            case .enableNotifications,
                 .accountActivation,
                 .accountBlockedMultiSignature,
                 .onboarding:
                Task {
                    try await bannerService.handleAction(key: action.banner.gemKey, action: action.type.gemAction)
                }
            case .suspiciousAsset: break
            case .tradePerpetuals:
                UIApplication.shared.open(DeepLink.perpetuals.gemUrl)
                preferences.isPerpetualEnabled = true
            }
        case let .button(bannerButton):
            switch bannerButton {
            case .buy: onSelectHeader(.buy)
            case .receive: onSelectHeader(.receive)
            }
        case .closeBanner:
            Task {
                try await bannerService.handleAction(key: action.banner.gemKey, action: action.type.gemAction)
            }
        }
        onSelect(url: action.url)
    }

    internal func onSelectEarn() {
        isPresentingSelectedAssetInput.wrappedValue = SelectedAssetInput(
            type: .earn(assetData.asset),
            assetData: assetData,
        )
    }

    private func onSelectBuy() {
        onSelectHeader(.buy)
    }

    private func onSelectSwap() {
        onSelectHeader(.swap)
    }

    func onSelectShareAsset() {
        isPresentingAssetSheet = .share
    }

    func onTransferComplete() {
        isPresentingAssetSheet = .none
    }

    func onTogglePriceAlert() {
        Task {
            let enabled = !assetData.isPriceAlertsEnabled
            do {
                try await setPriceAlert(enabled: enabled)
                isPresentingToastMessage = .priceAlert(for: assetData.asset.name, enabled: enabled)
            } catch {
                debugLog("onTogglePriceAlert error \(error)")
            }
        }
    }

    func onSelectTokenStatus() {
        isPresentingAssetSheet = .info(.assetStatus(scoreViewModel.scoreType))
    }

    func onSelectPendingUnconfirmedInfo() {
        isPresentingAssetSheet = .info(.pendingUnconfirmedBalance)
    }

    func onSelectPin() {
        let pinned = !assetData.metadata.isPinned
        Task {
            do {
                try await assetsEnabler.pinAsset(wallet: wallet, assetId: asset.id, pinned: pinned)
                isPresentingToastMessage = .pin(asset.name, pinned: pinned)
            } catch {
                debugLog("onSelectPin error: \(error)")
            }
        }
    }

    func onSelectEnable() {
        Task {
            let enabled = !assetData.metadata.isBalanceEnabled
            do {
                try await assetsEnabler.enableAssets(wallet: wallet, assetIds: [asset.id], enabled: enabled)
                isPresentingToastMessage = .showAsset(visible: enabled)
            } catch {
                debugLog("onSelectEnable error: \(error)")
            }
        }
    }
}

// MARK: - Private

extension AssetSceneViewModel {
    private var addressExplorerUrl: URL {
        addressLink.url
    }

    private var viewAddressOnTitle: String {
        Localized.Asset.viewAddressOn(addressLink.name)
    }

    private var viewTokenOnTitle: String? {
        if let link = tokenLink {
            return Localized.Asset.viewTokenOn(link.name)
        }
        return .none
    }

    private var tokenExplorerUrl: URL? {
        tokenLink?.url
    }

    private var tokenLink: BlockExplorerLink? {
        guard let tokenId = assetModel.asset.tokenId else {
            return .none
        }
        return explorerService.getTokenUrl(chain: assetModel.asset.chain.rawValue, address: tokenId).map { BlockExplorerLink($0) }
    }

    private static let showStakedBalanceTypes: [Primitives.BalanceType] = [.staked, .pending, .rewards]

    private var addressLink: BlockExplorerLink {
        BlockExplorerLink(explorerService.getAddressUrl(chain: assetModel.asset.chain.rawValue, address: assetDataModel.address))
    }

    private var feeAssetDataModel: AssetDataViewModel {
        AssetDataViewModel(
            assetData: chainAssetData.feeAssetData,
            formatter: .auto,
            currencyCode: preferences.preferences.currency,
        )
    }

    private func onSelect(url: URL?) {
        guard let url else { return }
        isPresentingAssetSheet = .url(url)
    }

    private func fetchTransactions() async {
        do {
            try await transactionsService.sync(walletId: walletModel.wallet.id.id, assetId: assetModel.asset.id.identifier)
        } catch {
            // TODO: - handle fetchTransactions error
            debugLog("asset scene: fetchTransactions error \(error)")
        }
    }

    private func setPriceAlert(enabled: Bool) async throws {
        let currency = try Currency(id: preferences.preferences.currency)
        let priceAlert = PriceAlert.default(for: assetModel.asset.id, currency: currency)
        if enabled {
            try await priceAlertService.enable(priceAlert: priceAlert)
        } else {
            try await priceAlertService.delete(priceAlerts: [priceAlert])
        }
    }

    private func updateAsset() async {
        async let asset: Void = updateAssetData()
        async let prices: Void = updatePrices()
        _ = await (asset, prices)
    }

    private func updateAssetData() async {
        let associations: [AssetAssociation]
        do {
            let asset = try await assetsService.syncAsset(
                for: assetModel.asset.id,
                currency: preferences.preferences.currency,
            )
            associations = asset.associations
        } catch {
            // TODO: - handle updateAsset error
            debugLog("asset scene: updateAsset error \(error)")
            return
        }

        do {
            try await assetsService.prefetchAssets(for: associations.map(\.assetId))
        } catch {
            debugLog("asset scene: prefetch associations error \(error)")
        }
    }

    private func updatePrices() async {
        do {
            try await priceUpdater.addPrices(assetIds: [assetModel.asset.id])
        } catch {
            debugLog("asset scene: addPrices error \(error)")
        }
    }

    private func updateWallet() async {
        async let balance: Void? = try? balanceService.update(walletId: walletModel.wallet.id.id, assetIds: [assetModel.asset.id.identifier])
        async let transactions: Void = fetchTransactions()
        _ = await (balance, transactions)
    }

    private func updatePriceAlerts() async {
        do {
            try await priceAlertService.update(assetId: asset.id.identifier)
        } catch {
            debugLog("asset scene: price alerts update error \(error)")
        }
    }
}
