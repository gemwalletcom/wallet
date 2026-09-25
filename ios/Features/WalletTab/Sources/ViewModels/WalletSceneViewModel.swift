// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import enum Gemstone.GemHeaderButtonKind
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

    public let collectionsModel: CollectionsViewModel

    public var wallet: Wallet {
        walletQuery.value.wallet
    }

    // db queries
    public let walletQuery: ObservableQuery<MappedRequest<WalletRequest, WalletEntry>>
    public let fiatValuesQuery: ObservableQuery<AssetFiatValuesRequest>
    public let perpetualBalanceQuery: ObservableQuery<PerpetualWalletBalanceRequest>
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

        walletQuery = ObservableQuery(MappedRequest(WalletRequest(walletId: wallet.id), transform: WalletEntry.init(wallet:)), initialValue: WalletEntry(wallet: wallet))
        fiatValuesQuery = ObservableQuery(
            AssetFiatValuesRequest(walletId: wallet.id),
            initialValue: [],
        )
        perpetualBalanceQuery = ObservableQuery(
            PerpetualWalletBalanceRequest(walletId: wallet.id, assetId: Chain.hyperCore.defaultAsset(type: .perpetual).id),
            initialValue: nil,
        )
        assetsQuery = ObservableQuery(AssetsRequest(walletId: wallet.id, filters: [.enabledBalance], limit: nil), initialValue: [])
        bannersQuery = ObservableQuery(
            BannersRequest(walletId: wallet.id, assetId: .none, events: GemConstants.walletBannerEvents),
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
            header: WalletHeaderViewModel(state: viewState),
            showPerpetuals: viewState.showsPerpetuals,
            showCollections: viewState.showCollections,
            banner: viewState.banner,
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

    internal func onHeaderAction(type: GemHeaderButtonKind) {
        switch type {
        case .buy: isPresentingSheet = .selectAsset(.buy, chains: [])
        case .send: isPresentingSheet = .selectAsset(.send(.none), chains: [])
        case .receive: isPresentingSheet = .selectAsset(.receive(.asset), chains: [])
        case .swap: isPresentingSheet = .swap
        case .more, .deposit, .withdraw: break
        }
    }

    internal func onSelectWatchWalletInfo() {
        isPresentingSheet = .infoSheet(.watchWallet)
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
        service.showsInitialLoading()
    }

    func setAssetPinned(_ assetId: AssetId, pinned: Bool) async throws {
        try await service.setAssetPinned(assetId: assetId, pinned: pinned)
    }

    func setAssetsEnabled(_ assetIds: [AssetId], enabled: Bool) async throws {
        try await service.setAssetsEnabled(assetIds: assetIds, enabled: enabled)
    }

    var assetItems: ListAssetItemsViewModel {
        ListAssetItemsViewModel(currency: observablePreferences.currency, rowStyle: service.assetRowStyle())
    }
}
