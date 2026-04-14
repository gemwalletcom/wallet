// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import BannerService
import Components
import DiscoverAssetsService
import Formatters
import Foundation
import InfoSheet
import Localization
import Preferences
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI
import WalletService

@Observable
@MainActor
public final class WalletSceneViewModel: Sendable {
    private let assetDiscoveryService: any AssetDiscoverable
    private let balanceService: BalanceService
    private let bannerService: BannerService
    private let walletService: WalletService

    let observablePreferences: ObservablePreferences

    public var wallet: Wallet

    // db queries
    public let totalFiatQuery: ObservableQuery<TotalValueRequest>
    public let assetsQuery: ObservableQuery<AssetsRequest>
    public let bannersQuery: ObservableQuery<BannersRequest>

    // db observed values
    public var totalFiatValue: TotalFiatValue { totalFiatQuery.value }
    public var assets: [AssetData] { assetsQuery.value }
    public var banners: [Banner] { bannersQuery.value }

    public var isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>
    public var isPresentingSheet: WalletSheetType?
    public var isPresentingSearch = false
    public var isPresentingUrl: URL?
    public var isPresentingToastMessage: ToastMessage?

    public var isLoadingAssets = false

    public init(
        assetDiscoveryService: any AssetDiscoverable,
        balanceService: BalanceService,
        bannerService: BannerService,
        walletService: WalletService,
        observablePreferences: ObservablePreferences,
        wallet: Wallet,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
    ) {
        self.wallet = wallet
        self.assetDiscoveryService = assetDiscoveryService
        self.balanceService = balanceService
        self.bannerService = bannerService
        self.walletService = walletService
        self.observablePreferences = observablePreferences

        totalFiatQuery = ObservableQuery(TotalValueRequest(walletId: wallet.walletId, balanceType: .wallet), initialValue: .zero)
        assetsQuery = ObservableQuery(AssetsRequest(walletId: wallet.walletId, filters: [.enabledBalance]), initialValue: [])
        bannersQuery = ObservableQuery(BannersRequest(walletId: wallet.walletId, assetId: .none, chain: .none, events: [.accountBlockedMultiSignature, .onboarding]), initialValue: [])
        self.isPresentingSelectedAssetInput = isPresentingSelectedAssetInput
    }

    public var currentWallet: Wallet? { walletService.currentWallet }

    var manageTokenTitle: String { Localized.Wallet.manageTokenList }
    var perpetualsTitle: String { Localized.Perpetuals.title }

    public var searchImage: Image { Images.System.search }
    public var manageImage: Image { Images.Actions.manage }

    var showPinnedSection: Bool {
        !sections.pinned.isEmpty
    }

    var showPerpetuals: Bool {
        observablePreferences.isPerpetualEnabled && wallet.isMultiCoins
    }

    var currencyCode: String {
        observablePreferences.preferences.currency
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

    var walletBannersModel: WalletSceneBannersViewModel {
        WalletSceneBannersViewModel(
            banners: banners,
            totalFiatValue: totalFiatValue.value,
        )
    }
}

// MARK: - Business Logic

public extension WalletSceneViewModel {
    internal func fetch() async {
        await updateWallet(for: wallet)
    }

    internal func fetchOnce() async {
        await fetchOnce(wallet: wallet)
    }

    func onSelectWalletBar() {
        isPresentingSheet = .wallets
    }

    func onSelectManage() {
        isPresentingSheet = .selectAsset(.manage)
    }

    func onToggleSearch() {
        isPresentingSearch.toggle()
    }

    func onSelectAddCustomToken() {
        isPresentingSheet = .addAsset
    }

    internal func onSelectPortfolio() {
        isPresentingSheet = .portfolio
    }

    internal func onHeaderAction(type: HeaderButtonType) {
        switch type {
        case .buy: isPresentingSheet = .selectAsset(.buy)
        case .send: isPresentingSheet = .selectAsset(.send)
        case .receive: isPresentingSheet = .selectAsset(.receive(.asset))
        case .sell, .swap, .more, .stake, .deposit, .withdraw: break
        }
    }

    internal func onCloseBanner(banner: Banner) {
        bannerService.onClose(banner)
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
            case .buy: isPresentingSheet = .selectAsset(.buy)
            case .receive: isPresentingSheet = .selectAsset(.receive(.asset))
            }
        }
        isPresentingUrl = action.url
    }

    internal func onHideAsset(_ assetId: AssetId) {
        do {
            try balanceService.hideAsset(walletId: wallet.walletId, assetId: assetId)
        } catch {
            debugLog("WalletSceneViewModel hide Asset error: \(error)")
        }
    }

    internal func onPinAsset(_ asset: Asset, value: Bool) {
        do {
            try balanceService.setPinned(value, walletId: wallet.walletId, assetId: asset.id)
            isPresentingToastMessage = .pin(asset.name, pinned: value)
        } catch {
            debugLog("WalletSceneViewModel pin asset error: \(error)")
        }
    }

    internal func onCopyAddress(_ message: String) {
        isPresentingToastMessage = .copy(message)
    }

    func onChangeWallet(_: Wallet?, _ newWallet: Wallet?) {
        guard let newWallet else { return }

        if wallet.walletId != newWallet.walletId {
            refresh(for: newWallet)
        } else if wallet != newWallet {
            wallet = newWallet
        }
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
    private func fetchOnce(wallet: Wallet) async {
        let shouldShowLoadingAssets = shouldShowInitialLoadingAssets(for: wallet)

        if shouldShowLoadingAssets {
            isLoadingAssets = true
        }

        await updateWallet(for: wallet)

        if shouldShowLoadingAssets, self.wallet.walletId == wallet.walletId {
            isLoadingAssets = false
        }
    }

    private func updateWallet(for wallet: Wallet) async {
        let assetIds = assets.map(\.asset.id)
        async let balance: () = balanceService.updateBalance(for: wallet, assetIds: assetIds)
        async let discovery: () = discoverAssets(wallet: wallet)
        _ = await (balance, discovery)
    }

    private func discoverAssets(wallet: Wallet) async {
        do {
            try await assetDiscoveryService.discoverAssets(wallet: wallet)
        } catch {
            debugLog("WalletSceneViewModel discoverAssets error: \(error)")
        }
    }

    private func shouldShowInitialLoadingAssets(for wallet: Wallet) -> Bool {
        let preferences = WalletPreferences(walletId: wallet.walletId)
        return !preferences.completeInitialLoadAssets && preferences.assetsTimestamp == .zero
    }

    private func refresh(for newWallet: Wallet) {
        isLoadingAssets = false
        wallet = newWallet
        totalFiatQuery.request.walletId = newWallet.walletId
        assetsQuery.request.walletId = newWallet.walletId
        bannersQuery.request.walletId = newWallet.walletId

        Task { await fetchOnce(wallet: newWallet) }
    }

    private func handleBanner(action: BannerAction) async throws {
        try await bannerService.handleAction(action)
    }
}
