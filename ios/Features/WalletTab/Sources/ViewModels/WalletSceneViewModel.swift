// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import BannerService
import Components
import DiscoverAssetsService
import Foundation
import GemstonePrimitives
import InfoSheet
import Localization
import NFT
import NFTService
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
    private let bannerService: BannerService
    private let walletService: WalletService

    let observablePreferences: ObservablePreferences

    public let assetsModel: WalletAssetsSectionViewModel
    public let collectionsModel: CollectionsViewModel

    var selectedContentType: WalletContentType = .assets

    public private(set) var wallet: Wallet

    public let totalFiatQuery: ObservableQuery<TotalValueRequest>
    public let bannersQuery: ObservableQuery<BannersRequest>

    public var isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>
    public var isPresentingSheet: WalletSheetType?
    public var isPresentingSearch = false
    public var isPresentingUrl: URL?
    public var isPresentingToastMessage: ToastMessage?

    public init(
        assetDiscoveryService: any AssetDiscoverable,
        balanceService: BalanceService,
        bannerService: BannerService,
        walletService: WalletService,
        nftService: NFTService,
        observablePreferences: ObservablePreferences,
        wallet: Wallet,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
    ) {
        self.wallet = wallet
        self.bannerService = bannerService
        self.walletService = walletService
        self.observablePreferences = observablePreferences
        assetsModel = WalletAssetsSectionViewModel(
            assetDiscoveryService: assetDiscoveryService,
            balanceService: balanceService,
            observablePreferences: observablePreferences,
            wallet: wallet,
        )
        collectionsModel = CollectionsViewModel(
            nftService: nftService,
            walletService: walletService,
            wallet: wallet,
        )

        totalFiatQuery = ObservableQuery(TotalValueRequest(walletId: wallet.id, type: .wallet), initialValue: .zero)
        bannersQuery = ObservableQuery(BannersRequest(walletId: wallet.id, assetId: .none, chain: .none, events: [.accountBlockedMultiSignature, .onboarding]), initialValue: [])
        self.isPresentingSelectedAssetInput = isPresentingSelectedAssetInput
    }

    public var currentWallet: Wallet? {
        walletService.currentWallet
    }

    public var searchImage: Image {
        Images.System.search
    }

    var perpetualsTitle: String {
        Localized.Perpetuals.title
    }

    public var walletBarModel: WalletBarViewViewModel {
        let walletModel = WalletViewModel(wallet: wallet)
        return WalletBarViewViewModel(
            name: walletModel.name,
            image: walletModel.avatarImage,
        )
    }

    var totalFiatValue: TotalFiatValue {
        totalFiatQuery.value
    }

    var banners: [Banner] {
        bannersQuery.value
    }

    var showPerpetuals: Bool {
        observablePreferences.showPerpetuals(for: wallet)
    }

    var currencyCode: String {
        observablePreferences.preferences.currency
    }

    var availableContentTypes: [WalletContentType] {
        WalletContentType.allCases.filter(isAvailable)
    }

    var showContentTypePicker: Bool {
        availableContentTypes.count > 1
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

    private func isAvailable(_ type: WalletContentType) -> Bool {
        switch type {
        case .assets: true
        case .collections: isSupported(\.isNFTSupported)
        case .defi: isSupported(\.isDefiSupported)
        }
    }

    private func isSupported(_ chainFlag: KeyPath<Chain, Bool>) -> Bool {
        switch wallet.type {
        case .multicoin: true
        case .single, .privateKey, .view:
            wallet.accounts.first?.chain[keyPath: chainFlag] ?? false
        }
    }
}

// MARK: - Business Logic

public extension WalletSceneViewModel {
    func onSelectWalletBar() {
        isPresentingSheet = .wallets
    }

    func onToggleSearch() {
        isPresentingSearch.toggle()
    }

    func onSelectAddCustomToken() {
        isPresentingSheet = .addAsset
    }

    func onChangeWallet(_: Wallet?, _ newWallet: Wallet?) {
        guard let newWallet else { return }

        if wallet.id != newWallet.id {
            refresh(for: newWallet)
        } else if wallet != newWallet {
            wallet = newWallet
            assetsModel.wallet = newWallet
            collectionsModel.wallet = newWallet
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

    func presentPriceAlert(_ asset: Asset) {
        isPresentingSheet = .setPriceAlert(asset)
    }
}

// MARK: - Internal

extension WalletSceneViewModel {
    func fetchOnce() async {
        async let assets: () = assetsModel.fetchOnce()
        async let collections: () = isAvailable(.collections) ? collectionsModel.fetch() : ()
        _ = await (assets, collections)
    }

    func refreshSelectedContent() async {
        switch selectedContentType {
        case .assets: await assetsModel.fetch()
        case .collections: await collectionsModel.fetch()
        case .defi: break
        }
    }

    func onSelectPortfolio() {
        isPresentingSheet = .portfolio(.wallet)
    }

    func onHeaderAction(type: HeaderButtonType) {
        switch type {
        case .buy: isPresentingSheet = .selectAsset(.buy)
        case .send: isPresentingSheet = .selectAsset(.send)
        case .receive: isPresentingSheet = .selectAsset(.receive(receiveType))
        case .sell, .swap, .more, .stake, .deposit, .withdraw: break
        }
    }

    func onSelectWatchWalletInfo() {
        isPresentingSheet = .infoSheet(.watchWallet)
    }

    func onBanner(action: BannerAction) {
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
}

// MARK: - Private

extension WalletSceneViewModel {
    private var receiveType: ReceiveAssetType {
        switch selectedContentType {
        case .assets, .defi: .asset
        case .collections: .collection
        }
    }

    private func refresh(for newWallet: Wallet) {
        wallet = newWallet
        totalFiatQuery.request.walletId = newWallet.id
        bannersQuery.request.walletId = newWallet.id

        assetsModel.refresh(for: newWallet)
        collectionsModel.onChangeWallet(collectionsModel.wallet, newWallet)
        selectedContentType = .assets
    }

    private func handleBanner(action: BannerAction) async throws {
        try await bannerService.handleAction(action)
    }
}
