// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Foundation
import GemstonePrimitives
import GemstoneServices
import MarketInsight
import NFT
import Primitives
import PrimitivesComponents
import Recents
import Store
import SwiftUI
import WalletTab
import class Gemstone.GemAddAssetService
import class Gemstone.GemAssetDetailsService
import class Gemstone.GemAssetSelectionService
import class Gemstone.GemChartService
import class Gemstone.GemWalletHomeService

extension ViewModelFactory {
    @MainActor
    public func assetScene(
        wallet: Wallet,
        asset: Asset,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
    ) -> AssetSceneViewModel {
        AssetSceneViewModel(
            service: Gemstone.GemAssetDetailsService(
                assets: assetsService,
                balances: balanceService,
                transactions: transactionsService,
                banners: bannerService,
                swap: swapService,
                explorer: explorerService,
                priceAlerts: priceAlertService,
                stream: streamSubscriptionService,
                deeplinks: deeplinkService,
                session: walletSessionService,
            ),
            preferences: observablePreferences,
            input: AssetSceneInput(wallet: wallet, asset: asset),
            isPresentingSelectedAssetInput: isPresentingSelectedAssetInput,
        )
    }

    @MainActor
    public func portfolioScene(wallet: Wallet, defaultType: PortfolioType) -> PortfolioSceneViewModel {
        PortfolioSceneViewModel(wallet: wallet, service: portfolioService, preferences: observablePreferences, defaultType: defaultType)
    }

    @MainActor
    public func walletScene(
        wallet: Wallet,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
        isPresentingWallets: Binding<Bool>,
    ) -> WalletSceneViewModel {
        WalletSceneViewModel(
            service: walletHomeService(),
            observablePreferences: observablePreferences,
            collectionsModel: CollectionsViewModel(service: nftService, wallet: wallet),
            wallet: wallet,
            isPresentingSelectedAssetInput: isPresentingSelectedAssetInput,
            isPresentingWallets: isPresentingWallets,
        )
    }

    @MainActor
    public func walletSearchScene(
        wallet: Wallet,
        onDismissSearch: VoidAction,
        onSelectAssetAction: AssetAction,
        onAddToken: VoidAction,
    ) -> WalletSearchSceneViewModel {
        WalletSearchSceneViewModel(
            wallet: wallet,
            service: assetSelectionService(),
            recentModel: RecentAssetsModel(walletId: wallet.id, types: RecentActivityType.allCases, service: recentAssetsService),
            onDismissSearch: onDismissSearch,
            onSelectAssetAction: onSelectAssetAction,
            onAddToken: onAddToken,
        )
    }

    @MainActor
    public func networkAssetsScene(wallet: Wallet, chain: Chain, onManageAssets: @escaping () -> Void) -> NetworkAssetsSceneViewModel {
        NetworkAssetsSceneViewModel(wallet: wallet, chain: chain, service: walletHomeService(), onManageAssets: onManageAssets)
    }

    private func walletHomeService() -> GemWalletHomeService {
        GemWalletHomeService(
            balances: balanceService,
            discovery: assetDiscoveryService,
            banners: bannerService,
            walletPreferences: walletPreferencesService,
            preferences: preferencesService,
            session: walletSessionService,
        )
    }

    private func assetSelectionService() -> GemAssetSelectionService {
        GemAssetSelectionService(
            search: searchService,
            balances: balanceService,
            priceAlerts: priceAlertService,
            recentActivity: recentAssetsService,
            preferences: preferencesService,
            perpetuals: perpetualService,
            session: walletSessionService,
            swap: swapService,
        )
    }

    @MainActor
    public func chartScene(
        asset: Asset,
        walletId: WalletId,
        onSetPriceAlert: @escaping (Asset) -> Void,
    ) -> ChartSceneViewModel {
        ChartSceneViewModel(
            service: Gemstone.GemChartService(
                api: apiClient,
                price: priceService,
                preferences: preferencesService,
                priceAlerts: priceAlertService,
                explorer: explorerService,
            ),
            assetModel: AssetViewModel(asset: asset),
            walletId: walletId,
            onSetPriceAlert: onSetPriceAlert,
        )
    }

    @MainActor
    public func addAssetScene(wallet: Wallet) -> AddAssetSceneViewModel {
        AddAssetSceneViewModel(
            wallet: wallet,
            service: Gemstone.GemAddAssetService(assets: assetsService, balances: balanceService, explorer: explorerService),
        )
    }

    @MainActor
    public func selectAssetScene(selectType: SelectAssetType, selectAssetAction: AssetAction = .none) -> SelectAssetViewModel? {
        currentWallet(in: currentWallets()).map { selectAssetScene(wallet: $0, selectType: selectType, selectAssetAction: selectAssetAction) }
    }

    @MainActor
    public func selectAssetScene(
        wallet: Wallet,
        selectType: SelectAssetType,
        selectAssetAction: AssetAction = .none,
        chains: [Chain] = [],
    ) -> SelectAssetViewModel {
        SelectAssetViewModel(
            wallet: wallet,
            selectType: selectType,
            service: assetSelectionService(),
            recentAssetsService: recentAssetsService,
            selectAssetAction: selectAssetAction,
            chains: chains,
        )
    }

    @MainActor
    public func assetsResultsScene(
        wallet: Wallet,
        request: WalletSearchRequest,
        title: String,
        onSelectAsset: @escaping (Asset) -> Void,
    ) -> AssetsResultsSceneViewModel {
        AssetsResultsSceneViewModel(
            wallet: wallet,
            service: assetSelectionService(),
            request: request,
            title: title,
            onSelectAsset: onSelectAsset,
        )
    }
}
