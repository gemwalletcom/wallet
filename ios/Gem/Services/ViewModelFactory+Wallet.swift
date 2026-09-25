// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Foundation
import class Gemstone.GemAddAssetService
import class Gemstone.GemAssetDetailsService
import class Gemstone.GemAssetSelectionService
import class Gemstone.GemChartService
import class Gemstone.GemWalletHomeService
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

public extension ViewModelFactory {
    @MainActor
    func assetScene(
        wallet: Wallet,
        asset: Asset,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
        onSelectPerpetuals: VoidAction,
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
            onSelectPerpetuals: onSelectPerpetuals,
        )
    }

    @MainActor
    func addressDetailsScene(chainAddress: ChainAddress) -> AddressDetailsSceneViewModel {
        AddressDetailsSceneViewModel(
            chainAddress: chainAddress,
            service: addressDetailsService,
        )
    }

    @MainActor
    func portfolioScene(wallet: Wallet, defaultType: PortfolioType) -> PortfolioSceneViewModel {
        PortfolioSceneViewModel(wallet: wallet, service: portfolioService, preferences: observablePreferences, defaultType: defaultType)
    }

    @MainActor
    func walletScene(
        wallet: Wallet,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
        isPresentingWallets: Binding<Bool>,
    ) -> WalletSceneViewModel {
        WalletSceneViewModel(
            service: walletHomeService(),
            observablePreferences: observablePreferences,
            collectionsModel: collectionsScene(wallet: wallet),
            wallet: wallet,
            isPresentingSelectedAssetInput: isPresentingSelectedAssetInput,
            isPresentingWallets: isPresentingWallets,
        )
    }

    @MainActor
    func walletSearchScene(
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
    func networkAssetsScene(wallet: Wallet, chain: Chain, onManageAssets: @escaping () -> Void) -> NetworkAssetsSceneViewModel {
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
            assets: assetsService,
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
    func chartScene(
        asset: Asset,
        onSetPriceAlert: @escaping (Asset) -> Void,
        onSelectAddress: @escaping @MainActor @Sendable (ChainAddress) -> Void,
    ) -> ChartSceneViewModel {
        ChartSceneViewModel(
            service: Gemstone.GemChartService(
                api: apiClient,
                price: priceService,
                preferences: preferencesService,
                explorer: explorerService,
            ),
            preferences: observablePreferences,
            assetModel: AssetViewModel(asset: asset),
            onSetPriceAlert: onSetPriceAlert,
            onSelectAddress: onSelectAddress,
        )
    }

    @MainActor
    func addAssetScene(wallet: Wallet) -> AddAssetSceneViewModel {
        AddAssetSceneViewModel(
            wallet: wallet,
            service: Gemstone.GemAddAssetService(assets: assetsService, balances: balanceService, explorer: explorerService),
        )
    }

    @MainActor
    func selectAssetScene(selectType: SelectAssetType, selectAssetAction: AssetAction = .none) -> SelectAssetViewModel? {
        currentWallet(in: currentWallets()).map { selectAssetScene(wallet: $0, selectType: selectType, selectAssetAction: selectAssetAction) }
    }

    @MainActor
    func selectAssetScene(
        wallet: Wallet,
        selectType: SelectAssetType,
        selectAssetAction: AssetAction = .none,
        chains: [Chain] = [],
    ) -> SelectAssetViewModel {
        SelectAssetViewModel(
            wallet: wallet,
            selectType: selectType,
            service: assetSelectionService(),
            paymentService: paymentService,
            recentAssetsService: recentAssetsService,
            selectAssetAction: selectAssetAction,
            chains: chains,
        )
    }

    @MainActor
    func assetsResultsScene(
        wallet: Wallet,
        request: WalletSearchQuery,
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
