// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemWalletPreferencesServiceProtocol
import protocol Gemstone.GemBalanceServiceProtocol
import struct Gemstone.GemBannerContent
import struct Gemstone.GemBannerContext
import protocol Gemstone.GemBannerServiceProtocol
import protocol Gemstone.GemNftServiceProtocol
import GemstoneServices
import Components
import Formatters
import Foundation
import protocol Gemstone.GemAssetDiscoveryServiceProtocol
import GemstonePrimitives
import InfoSheet
import Localization
import NFT
import Preferences
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class WalletSceneViewModel: Sendable, AssetActions {
    private let assetDiscoveryService: any GemAssetDiscoveryServiceProtocol
    let balanceService: any GemBalanceServiceProtocol
    private let bannerService: any GemBannerServiceProtocol
    private let walletPreferencesService: any GemWalletPreferencesServiceProtocol
    private let balanceCalculator = BalanceCalculator()

    let observablePreferences: ObservablePreferences

    public let collectionsModel: CollectionsViewModel

    public var wallet: Wallet {
        walletQuery.value
    }

    // db queries
    public let walletQuery: ObservableQuery<WalletRequest>
    public let fiatValuesQuery: ObservableQuery<AssetFiatValuesRequest>
    public let assetsQuery: ObservableQuery<AssetsRequest>
    public let bannersQuery: ObservableQuery<BannersRequest>

    public var isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>
    public var isPresentingScanner = false
    public var isPresentingWallets: Binding<Bool>
    public var isPresentingSheet: WalletSheetType?
    public var isPresentingSearch = false
    public var isPresentingUrl: URL?
    public var isPresentingToastMessage: ToastMessage?

    public var isLoadingAssets = false

    public init(
        assetDiscoveryService: any GemAssetDiscoveryServiceProtocol,
        balanceService: any GemBalanceServiceProtocol,
        bannerService: any GemBannerServiceProtocol,
        nftService: any GemNftServiceProtocol,
        walletPreferencesService: any GemWalletPreferencesServiceProtocol,
        observablePreferences: ObservablePreferences,
        wallet: Wallet,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
        isPresentingWallets: Binding<Bool>,
    ) {
        self.assetDiscoveryService = assetDiscoveryService
        self.balanceService = balanceService
        self.bannerService = bannerService
        self.walletPreferencesService = walletPreferencesService
        self.observablePreferences = observablePreferences
        collectionsModel = CollectionsViewModel(
            nftService: nftService,
            wallet: wallet,
        )

        walletQuery = ObservableQuery(WalletRequest(walletId: wallet.id), initialValue: wallet)
        fiatValuesQuery = ObservableQuery(
            AssetFiatValuesRequest(
                walletId: wallet.id,
                type: .wallet,
                perpetualAssetId: Chain.hyperCore.defaultAsset(type: .perpetual).id,
                includesPerpetualCollateral: walletPreferencesService.includesPerpetualCollateral(walletId: wallet.id.id),
            ),
            initialValue: [],
        )
        assetsQuery = ObservableQuery(AssetsRequest(walletId: wallet.id, filters: [.enabledBalance]), initialValue: [])
        bannersQuery = ObservableQuery(BannersRequest(walletId: wallet.id, assetId: .none, events: [.accountBlockedMultiSignature, .onboarding]), initialValue: [])
        self.isPresentingSelectedAssetInput = isPresentingSelectedAssetInput
        self.isPresentingWallets = isPresentingWallets
    }

    public var totalFiatValue: TotalFiatValue {
        balanceCalculator.totalFiatValue(fiatValuesQuery.value)
    }

    public var assets: [AssetData] {
        assetsQuery.value
    }

    public var banners: [Banner] {
        bannersQuery.value
    }

    var manageTokenTitle: String {
        Localized.Wallet.manageTokenList
    }

    var perpetualsTitle: String {
        Localized.Perpetuals.title
    }

    var collectionsTitle: String {
        Localized.Nft.collections
    }

    var collectionsContent: CollectionsContent {
        collectionsModel.content
    }

    public var searchImage: Image {
        Images.System.search
    }

    public var scannerImage: Image {
        Images.System.qrCodeViewfinder
    }

    public var manageImage: Image {
        Images.Actions.manage
    }

    var showPinnedSection: Bool {
        !sections.pinned.isEmpty
    }

    var showPerpetuals: Bool {
        observablePreferences.showPerpetuals(for: wallet)
    }

    var showCollections: Bool {
        switch wallet.type {
        case .multicoin: true
        case .single, .privateKey, .view:
            wallet.accounts.first?.chain.isNFTSupported ?? false
        }
    }

    var currencyCode: String {
        observablePreferences.currency
    }

    var sections: AssetsSections {
        AssetsSections.from(assets)
    }

    public var walletBarModel: WalletBarViewViewModel {
        let walletModel = WalletViewModel(wallet: wallet)
        return WalletBarViewViewModel(
            name: walletModel.name,
            image: walletModel.avatarImage,
        )
    }

    var walletHeaderModel: WalletHeaderViewModel {
        WalletHeaderViewModel(
            walletType: wallet.type,
            totalValue: totalFiatValue,
            currencyCode: currencyCode,
            bannerEventsViewModel: HeaderBannerEventViewModel(events: banners.map(\.event)),
        )
    }

    var visibleBanners: [Banner] {
        (try? bannerService.visibleBanners(banners, walletId: wallet.id, asset: .none, context: bannerContext)) ?? []
    }

    func bannerContent(for banner: Banner) -> GemBannerContent {
        bannerService.content(for: banner)
    }

    private var bannerContext: GemBannerContext {
        GemBannerContext(
            hasWallet: true,
            hasAsset: false,
            isStakeable: false,
            hasStakeBalance: false,
            hasAvailableBalance: false,
            isAssetActivated: true,
            assetRankScore: .none,
            hasPerpetualsSupport: wallet.hasPerpetualsSupport,
            isWalletEmpty: totalFiatValue.value.isZero,
        )
    }
}

// MARK: - Business Logic

public extension WalletSceneViewModel {
    internal func load() async {
        await updateWallet(for: wallet)
    }

    internal func loadOnce() async {
        await loadOnce(wallet: wallet)
    }

    func onSelectWalletBar() {
        isPresentingWallets.wrappedValue = true
    }

    func onSelectManage(chains: [Chain] = []) {
        isPresentingSheet = .selectAsset(.manage, chains: chains)
    }

    func onToggleSearch() {
        isPresentingSearch.toggle()
    }

    func onSelectScanner() {
        isPresentingScanner = true
    }

    func onSelectAddCustomToken() {
        isPresentingSheet = .addAsset
    }

    internal func onSelectPortfolio() {
        isPresentingSheet = .portfolio(.wallet)
    }

    internal func onHeaderAction(type: HeaderButtonType) {
        switch type {
        case .buy: isPresentingSheet = .selectAsset(.buy, chains: [])
        case .send: isPresentingSheet = .selectAsset(.send(.none), chains: [])
        case .receive: isPresentingSheet = .selectAsset(.receive(.asset), chains: [])
        case .sell, .swap, .more, .stake, .deposit, .withdraw: break
        }
    }

    internal func onCloseBanner(banner: Banner) {
        Task { try await bannerService.close(key: banner.gemKey) }
    }

    internal func onSelectWatchWalletInfo() {
        isPresentingSheet = .infoSheet(.watchWallet)
    }

    internal func onBanner(action: BannerAction) {
        switch action.type {
        case .event, .closeBanner:
            Task {
                try await handleBanner(action: action)
            }
        case let .button(bannerButton):
            switch bannerButton {
            case .buy: isPresentingSheet = .selectAsset(.buy, chains: [])
            case .receive: isPresentingSheet = .selectAsset(.receive(.asset), chains: [])
            }
        }
        isPresentingUrl = action.url
    }

    internal func onCopyAddress(_ message: String) {
        isPresentingToastMessage = .copy(message)
    }

    func onWalletTabReselected(_: Bool, _: Bool) {
        isPresentingSearch = false
    }

    func onTransferComplete() {
        isPresentingSheet = nil
    }

    func onSetPriceAlertComplete(message: String) {
        isPresentingSheet = nil
        isPresentingToastMessage = .priceAlert(message: message)
    }

    func presentTransferData(_ data: TransferData) {
        isPresentingSheet = .transferData(data)
    }

    func presentPerpetualRecipientData(_ data: PerpetualRecipientData) {
        isPresentingSheet = .perpetualRecipientData(data)
    }

    func presentPriceAlert(_ asset: Asset) {
        isPresentingSheet = .setPriceAlert(asset)
    }
}

// MARK: - Private

extension WalletSceneViewModel {
    private func loadOnce(wallet: Wallet) async {
        let shouldShowLoadingAssets = shouldShowInitialLoadingAssets(for: wallet)

        if shouldShowLoadingAssets {
            isLoadingAssets = true
        }

        await updateWallet(for: wallet)

        if shouldShowLoadingAssets, self.wallet.id == wallet.id {
            isLoadingAssets = false
        }
    }

    private func updateWallet(for wallet: Wallet) async {
        let assetIds = assets.map(\.asset.id)
        async let balance: Void? = try? balanceService.update(walletId: wallet.id.id, assetIds: assetIds.ids)
        async let discovery: () = discoverAssets(wallet: wallet)
        _ = await (balance, discovery)
    }

    private func discoverAssets(wallet: Wallet) async {
        do {
            _ = try await assetDiscoveryService.discover(walletId: wallet.id.id)
        } catch {
            debugLog("WalletSceneViewModel discoverAssets error: \(error)")
        }
    }

    private func shouldShowInitialLoadingAssets(for wallet: Wallet) -> Bool {
        let completed = (try? walletPreferencesService.isInitialLoadCompleted(walletId: wallet.id, step: .assets)) ?? false
        let timestamp = walletPreferencesService.getAssetsTimestamp(walletId: wallet.id)
        return !completed && timestamp == 0
    }

    private func handleBanner(action: BannerAction) async throws {
        try await bannerService.applyAction(key: action.banner.gemKey, action: action.type.gemAction)
    }
}
