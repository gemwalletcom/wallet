// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Components
import Formatters
import Foundation
import enum Gemstone.GemHeaderButtonTap
import enum Gemstone.GemInfoTopic
import struct Gemstone.GemPerpetualCollateral
import enum Gemstone.GemServiceError
import protocol Gemstone.GemWalletHomeServiceProtocol
import GemstonePrimitives
import GemstoneServices
import InfoSheet
import Localization
import NFT
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

    public let collectionsModel: CollectionsSceneViewModel

    public var wallet: Wallet {
        walletQuery.value.wallet
    }

    // db queries
    public let walletQuery: ObservableQuery<MappedQuery<WalletQuery, WalletEntry>>
    public let fiatValuesQuery: ObservableQuery<AssetFiatValuesQuery>
    public let perpetualBalanceQuery: ObservableQuery<PerpetualWalletBalanceQuery>
    public let assetsQuery: ObservableQuery<AssetsQuery>
    public let bannersQuery: ObservableQuery<BannersQuery>

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
        collectionsModel: CollectionsSceneViewModel,
        wallet: Wallet,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
        isPresentingWallets: Binding<Bool>,
    ) {
        self.service = service
        self.observablePreferences = observablePreferences
        self.collectionsModel = collectionsModel

        walletQuery = ObservableQuery(MappedQuery(WalletQuery(walletId: wallet.id), transform: WalletEntry.init(wallet:)), initialValue: WalletEntry(wallet: wallet))
        fiatValuesQuery = ObservableQuery(
            AssetFiatValuesQuery(walletId: wallet.id),
            initialValue: [],
        )
        perpetualBalanceQuery = ObservableQuery(
            PerpetualWalletBalanceQuery(walletId: wallet.id, assetId: Chain.hyperCore.defaultAsset(type: .perpetual).id),
            initialValue: nil,
        )
        assetsQuery = ObservableQuery(AssetsQuery(walletId: wallet.id, filters: [.enabledBalance], limit: nil), initialValue: [])
        bannersQuery = ObservableQuery(
            BannersQuery(walletId: wallet.id, assetId: .none, events: GemConstants.walletBannerEvents),
            initialValue: [],
        )
        self.isPresentingSelectedAssetInput = isPresentingSelectedAssetInput
        self.isPresentingWallets = isPresentingWallets
    }

    public var assets: [AssetData] {
        assetsQuery.value
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

    public var walletBarModel: WalletBarViewViewModel {
        let row = walletQuery.value.row
        return WalletBarViewViewModel(
            name: row.name,
            image: row.avatarImage,
        )
    }

    var homeState: WalletHomeState {
        let viewState = service.viewState(
            wallet: wallet,
            balances: fiatValuesQuery.value,
            perpetual: perpetualCollateral,
            banners: bannersQuery.value,
        )
        return WalletHomeState(
            sections: AssetsSections.from(assets),
            header: viewState.header.valueHeader,
            showPerpetuals: viewState.showsPerpetuals,
            showCollections: viewState.showCollections,
            banner: viewState.banner,
        )
    }
}

// MARK: - Business Logic

public extension WalletSceneViewModel {
    internal func load() async {
        await updateWallet(wallet: wallet)
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

    internal func onHeaderAction(_ tap: GemHeaderButtonTap) {
        switch tap {
        case .buy: isPresentingSheet = .selectAsset(.buy, chains: [])
        case .send: isPresentingSheet = .selectAsset(.send(.none), chains: [])
        case .receive: isPresentingSheet = .selectAsset(.receive(.asset), chains: [])
        case .swap: isPresentingSheet = .swap
        case .deposit, .withdraw, .sendCollectible, .collectibleMenu: break
        }
    }

    internal func onSelectWatchWalletInfo() {
        isPresentingSheet = .infoSheet(GemInfoTopic.watchWallet.infoSheet)
    }

    internal func onBanner(action: BannerAction) {
        switch action.type {
        case let .destination(destination):
            switch destination {
            case let .url(url): isPresentingUrl = URL(string: url)
            case .stake, .activateAsset, .perpetuals: break
            }
        case .closeBanner:
            Task {
                do {
                    try await service.closeBanner(key: action.key)
                } catch let error as GemServiceError {
                    isPresentingToastMessage = .error(error.text().text)
                } catch {
                    isPresentingToastMessage = .error(Localized.Errors.errorOccurred)
                }
            }
        case let .button(bannerButton):
            switch bannerButton {
            case .buy: isPresentingSheet = .selectAsset(.buy, chains: [])
            case .receive: isPresentingSheet = .selectAsset(.receive(.asset), chains: [])
            }
        }
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
    private var perpetualCollateral: GemPerpetualCollateral? {
        perpetualBalanceQuery.value.map { GemPerpetualCollateral(balance: $0.balance.toGem(), price: $0.price) }
    }

    private func loadOnce(wallet: Wallet) async {
        isLoadingAssets = service.showsInitialLoading()
        await updateWallet(wallet: wallet)
    }

    private func updateWallet(wallet: Wallet) async {
        do {
            try await service.refresh()
        } catch {
            debugLog("WalletSceneViewModel refresh error: \(error)")
        }
        guard self.wallet.id == wallet.id else { return }
        isLoadingAssets = service.showsInitialLoading()
    }

    func setAssetPinned(_ assetId: AssetId, pinned: Bool) async throws {
        try await service.setAssetPinned(assetId: assetId, pinned: pinned)
    }

    func setAssetsEnabled(_ assetIds: [AssetId], enabled: Bool) async throws {
        try await service.setAssetsEnabled(assetIds: assetIds, enabled: enabled)
    }

    var assetItems: ListAssetItemsViewModel {
        ListAssetItemsViewModel(currency: observablePreferences.currency)
    }
}
