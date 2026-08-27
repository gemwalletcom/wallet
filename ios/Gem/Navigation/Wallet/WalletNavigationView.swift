// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import GemstoneServices
import Components
import InfoSheet
import Localization
import MarketInsight
import NFT
import Perpetuals
import PriceAlerts
import Primitives
import PrimitivesComponents
import QRScanner
import Store
import SwiftUI
import Transactions
import Transfer
import WalletTab

struct WalletNavigationView: View {
    @Environment(\.assetsEnabler) private var assetsEnabler
    @Environment(\.explorerService) private var explorerService
    @Environment(\.balanceService) private var balanceService
    @Environment(\.navigationHandler) private var navigationHandler
    @Environment(\.navigationState) private var navigationState
    @Environment(\.navigationPresenter) private var presenter
    @Environment(\.priceService) private var priceService
    @Environment(\.priceStore) private var priceStore
    @Environment(\.chartService) private var chartService
    @Environment(\.portfolioService) private var portfolioService
    @Environment(\.priceAlertService) private var priceAlertService
    @Environment(\.assetsService) private var assetsService
    @Environment(\.transactionsService) private var transactionsService
    @Environment(\.bannerService) private var bannerService
    @Environment(\.streamSubscriptionService) private var streamSubscriptionService
    @Environment(\.perpetualService) private var perpetualService
    @Environment(\.hyperliquidObserverService) private var hyperliquidObserverService
    @Environment(\.recentActivityStore) private var recentActivityStore
    @Environment(\.searchService) private var searchService
    @Environment(\.viewModelFactory) private var viewModelFactory
    @Environment(\.avatarService) private var avatarService
    @Environment(\.nftService) private var nftService
    @Environment(\.observablePreferences) private var preferences

    @State private var model: WalletSceneViewModel

    init(model: WalletSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        ZStack {
            WalletScene(model: model)
                .opacity(model.isPresentingSearch ? 0 : 1)

            if model.isPresentingSearch {
                WalletSearchScene(
                    model: WalletSearchSceneViewModel(
                        wallet: model.wallet,
                        searchService: searchService,
                        recentActivityStore: recentActivityStore,
                        assetsEnabler: assetsEnabler,
                        perpetualService: perpetualService,
                        onDismissSearch: model.onToggleSearch,
                        onSelectAssetAction: navigationState.openAsset,
                        onAddToken: model.onSelectAddCustomToken,
                    ),
                )
                .transition(.opacity)
            }
        }
        .onChange(of: navigationState.walletTabReselected, model.onWalletTabReselected)
        .bindQuery(model.walletQuery, model.assetsQuery, model.bannersQuery, model.fiatValuesQuery, model.collectionsModel.query)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            if !model.isPresentingSearch {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button(action: model.onSelectScanner) {
                        model.scannerImage
                    }
                    .accessibilityIdentifier("scan")
                }
                ToolbarItem(placement: .principal) {
                    WalletBarView(
                        model: model.walletBarModel,
                        action: model.onSelectWalletBar,
                    )
                    .liquidGlass()
                }
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: model.onToggleSearch) {
                        model.searchImage
                    }
                }
            }
        }
        .navigationDestination(for: Scenes.Asset.self) {
            AssetNavigationView(
                model: AssetSceneViewModel(
                    assetsEnabler: assetsEnabler,
                    balanceService: balanceService,
                    assetsService: assetsService,
                    transactionsService: transactionsService,
                    priceUpdater: streamSubscriptionService,
                    priceAlertService: priceAlertService,
                    bannerService: bannerService,
                    explorerService: explorerService,
                    input: AssetSceneInput(
                        wallet: model.wallet,
                        asset: $0.asset,
                    ),
                    isPresentingSelectedAssetInput: model.isPresentingSelectedAssetInput,
                ),
            )
        }
        .navigationDestination(for: Scenes.NetworkAssets.self) { destination in
            NetworkAssetsScene(
                model: NetworkAssetsSceneViewModel(
                    wallet: model.wallet,
                    chain: destination.chain,
                    balanceService: balanceService,
                    assetsEnabler: assetsEnabler,
                    preferences: preferences.preferences,
                    onManageAssets: { model.onSelectManage(chains: [destination.chain]) },
                ),
            )
        }
        .navigationDestination(for: Scenes.Transaction.self) {
            TransactionNavigationView(
                model: TransactionSceneViewModel(
                    transaction: $0.transaction,
                    walletId: model.wallet.id,
                    explorerService: explorerService,
                    onHeaderAction: onSelectTransactionHeaderAction,
                    onAddContact: { model.isPresentingSheet = .addContact($0) },
                ),
            )
        }
        .navigationDestination(for: Scenes.Collectible.self) {
            CollectibleScene(
                model: CollectibleViewModel(
                    wallet: model.wallet,
                    assetData: $0.assetData,
                    avatarService: avatarService,
                    nftService: nftService,
                    explorerService: explorerService,
                    isPresentingSelectedAssetInput: model.isPresentingSelectedAssetInput,
                ),
            )
        }
        .navigationDestination(for: Scenes.Collections.self) { _ in
            CollectionsSceneNavigationView(
                model: CollectionsViewModel(
                    nftService: nftService,
                    wallet: model.wallet,
                ),
            )
        }
        .navigationDestination(for: Scenes.Collection.self) { scene in
            CollectionsScene(
                model: CollectionViewModel(
                    wallet: model.wallet,
                    collectionId: scene.id,
                    collectionName: scene.name,
                ),
            )
        }
        .navigationDestination(for: Scenes.UnverifiedCollections.self) { _ in
            CollectionsScene(
                model: UnverifiedCollectionsViewModel(wallet: model.wallet),
            )
        }
        .navigationDestination(for: Scenes.Price.self) {
            ChartScene(
                model: ChartSceneViewModel(
                    explorerService: explorerService,
                    service: chartService,
                    priceStore: priceStore,
                    assetModel: AssetViewModel(asset: $0.asset),
                    priceAlertService: priceAlertService,
                    walletId: model.wallet.id,
                    onSetPriceAlert: model.presentPriceAlert,
                ),
            )
        }
        .navigationDestination(for: Scenes.Perpetuals.self) { _ in
            PerpetualsNavigationView(
                wallet: model.wallet,
                perpetualService: perpetualService,
                observerService: hyperliquidObserverService,
                recentActivityStore: recentActivityStore,
                onSelectAssetType: { model.isPresentingSheet = .selectAsset($0, chains: []) },
                onSelectAsset: navigationState.openAsset,
                onSelectPortfolio: { model.isPresentingSheet = .portfolio(.perpetuals) },
            )
        }
        .navigationDestination(for: Scenes.AssetsResults.self) { destination in
            AssetsResultsScene(
                model: viewModelFactory.assetsResultsScene(
                    wallet: model.wallet,
                    request: WalletSearchRequest(
                        walletId: model.wallet.id,
                        searchBy: destination.searchQuery,
                        scope: destination.scope,
                        limit: AssetsResultsSceneViewModel.defaultLimit,
                    ),
                    title: destination.title ?? Localized.Assets.title,
                    onSelectAsset: navigationState.openAsset,
                ),
            )
        }
        .navigationDestination(for: Scenes.Perpetual.self) {
            PerpetualNavigationView(
                asset: $0.asset,
                wallet: model.wallet,
                perpetualService: perpetualService,
                transactionsService: transactionsService,
                observerService: hyperliquidObserverService,
                explorerService: explorerService,
                isPresentingSheet: $model.isPresentingSheet,
            )
        }
        .navigationDestination(for: Scenes.AssetPriceAlert.self) {
            AssetPriceAlertsScene(
                model: AssetPriceAlertsViewModel(
                    priceAlertService: priceAlertService,
                    walletId: model.wallet.id,
                    asset: $0.asset,
                ),
            )
        }
        .scanQRCodeSheet(isPresented: $model.isPresentingScanner, action: onScan)
        .sheet(item: $model.isPresentingSheet) { sheet in
            Group {
                switch sheet {
                case let .selectAsset(type, chains):
                    SelectAssetSceneNavigationStack(
                        model: viewModelFactory.selectAssetScene(
                            wallet: model.wallet,
                            selectType: type,
                            chains: chains,
                        ),
                    )
                case let .infoSheet(type):
                    InfoSheetScene(type: type)
                case let .transferData(data):
                    ConfirmTransferNavigationStack(
                        wallet: model.wallet,
                        transferData: data,
                        onComplete: model.onTransferComplete,
                    )
                case let .perpetualRecipientData(data):
                    PerpetualPositionNavigationStack(
                        perpetualRecipientData: data,
                        wallet: model.wallet,
                        onComplete: { model.isPresentingSheet = nil },
                    )
                case let .setPriceAlert(asset):
                    SetPriceAlertNavigationStack(
                        model: SetPriceAlertViewModel(
                            walletId: model.wallet.id,
                            asset: asset,
                            priceAlertService: priceAlertService,
                        ) { model.onSetPriceAlertComplete(message: $0) },
                    )
                case .addAsset:
                    AddAssetNavigationStack(wallet: model.wallet)
                case let .portfolio(defaultType):
                    PortfolioScene(
                        model: PortfolioSceneViewModel(
                            wallet: model.wallet,
                            service: PortfolioDataService(
                                portfolioService: portfolioService,
                                perpetualService: perpetualService,
                                priceStore: priceStore,
                            ),
                            preferences: preferences,
                            defaultType: defaultType,
                        ),
                    )
                case let .addContact(action):
                    AddContactNavigationView(action: action)
                }
            }
            .id(sheet.id)
        }
        .safariSheet(url: $model.isPresentingUrl)
        .toast(message: $model.isPresentingToastMessage)
    }
}

// MARK: - Actions

extension WalletNavigationView {
    private func onScan(_ code: String) {
        Task { await navigationHandler.handle(code: code) }
    }

    private func onSelectTransactionHeaderAction(_ action: TransactionHeaderAction) {
        Task {
            do {
                try await presenter.handleTransactionHeaderAction(
                    action,
                    wallet: model.wallet,
                    navigationState: navigationState,
                    assetsService: assetsService,
                    nftService: nftService,
                    nftDestination: navigationState.wallet,
                )
            } catch {
                model.isPresentingToastMessage = .error(Localized.Errors.errorOccurred)
            }
        }
    }
}
