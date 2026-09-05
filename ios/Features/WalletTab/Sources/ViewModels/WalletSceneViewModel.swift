// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemBannerContent
import protocol Gemstone.GemWalletHomeServiceProtocol
import struct Gemstone.GemBannerContext
import GemstoneServices
import Components
import Formatters
import Foundation
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
    private let service: any GemWalletHomeServiceProtocol

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
        service: any GemWalletHomeServiceProtocol,
        observablePreferences: ObservablePreferences,
        collectionsModel: CollectionsViewModel,
        wallet: Wallet,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
        isPresentingWallets: Binding<Bool>,
    ) {
        self.service = service
        self.observablePreferences = observablePreferences
        self.collectionsModel = collectionsModel

        walletQuery = ObservableQuery(WalletRequest(walletId: wallet.id), initialValue: wallet)
        fiatValuesQuery = ObservableQuery(
            AssetFiatValuesRequest(
                walletId: wallet.id,
                type: .wallet,
                perpetualAssetId: Chain.hyperCore.defaultAsset(type: .perpetual).id,
                includesPerpetualCollateral: service.includesPerpetualCollateral(),
            ),
            initialValue: [],
        )
        assetsQuery = ObservableQuery(AssetsRequest(walletId: wallet.id, filters: [.enabledBalance]), initialValue: [])
        bannersQuery = ObservableQuery(BannersRequest(walletId: wallet.id, assetId: .none, events: [.accountBlockedMultiSignature, .onboarding]), initialValue: [])
        self.isPresentingSelectedAssetInput = isPresentingSelectedAssetInput
        self.isPresentingWallets = isPresentingWallets
    }

    public var totalFiatValue: TotalFiatValue {
        service.totalFiatValue(balances: fiatValuesQuery.value.map { $0.map() }).map()
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
        observablePreferences.showCollections(for: wallet)
    }

    var currencyCode: String {
        observablePreferences.currency.rawValue
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
            showsPnl: service.showsPnl(total: totalFiatValue.map()),
            buttons: service.headerButtons(wallet: wallet.map(), isEnabled: HeaderBannerEventViewModel(events: banners.map(\.event)).isButtonsEnabled),
        )
    }

    var visibleBanners: [Banner] {
        bannerContext.visibleBanners(banners, walletId: wallet.id, asset: .none)
    }

    func bannerContent(for banner: Banner) -> GemBannerContent {
        service.content(for: banner)
    }

    private var bannerContext: GemBannerContext {
        GemBannerContext(
            wallet: wallet.map(),
            hasAsset: false,
            isStakeable: false,
            hasStakeBalance: false,
            hasAvailableBalance: false,
            isAssetActivated: true,
            assetRankScore: .none,
            isWalletEmpty: assets.allSatisfy { $0.balance.total.isZero },
        )
    }
}

// MARK: - Business Logic

public extension WalletSceneViewModel {
    internal func load() async {
        await updateWallet()
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
        case .swap: isPresentingSheet = .swap
        case .sell, .more, .stake, .deposit, .withdraw: break
        }
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

}

// MARK: - Private

extension WalletSceneViewModel {
    private func loadOnce(wallet: Wallet) async {
        let shouldShowLoadingAssets = shouldShowInitialLoadingAssets

        if shouldShowLoadingAssets {
            isLoadingAssets = true
        }

        await updateWallet()

        if shouldShowLoadingAssets, self.wallet.id == wallet.id {
            isLoadingAssets = false
        }
    }

    private func updateWallet() async {
        do {
            try await service.refresh()
        } catch {
            debugLog("WalletSceneViewModel refresh error: \(error)")
        }
    }

    private var shouldShowInitialLoadingAssets: Bool {
        (try? service.showsInitialLoading()) ?? false
    }

    private func handleBanner(action: BannerAction) async throws {
        try await service.applyAction(action)
    }

    func setAssetPinned(_ assetId: AssetId, pinned: Bool) async throws {
        try await service.setAssetPinned(assetId: assetId, pinned: pinned)
    }

    func setAssetsEnabled(_ assetIds: [AssetId], enabled: Bool) async throws {
        try await service.setAssetsEnabled(assetIds: assetIds, enabled: enabled)
    }
}
