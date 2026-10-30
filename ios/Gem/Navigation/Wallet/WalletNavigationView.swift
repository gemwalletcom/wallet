// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Components
import GemstonePrimitives
import InfoSheet
import Localization
import NFT
import Perpetuals
import PriceAlerts
import Primitives
import PrimitivesComponents
import Store
import SwiftUI
import Transactions
import Wallet

struct WalletNavigationView: View {
    @Environment(\.navigationRouter) private var navigationRouter
    @Environment(\.navigationState) private var navigationState
    @Environment(\.navigationPresenter) private var presenter
    @Environment(\.viewModelFactory) private var viewModelFactory

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
                    model: viewModelFactory.walletSearchScene(
                        wallet: model.wallet,
                        onDismissSearch: model.onToggleSearch,
                        onSelectAssetAction: navigationRouter.openAsset,
                        onAddToken: model.onSelectAddCustomToken,
                    ),
                )
                .transition(.opacity)
            }
        }
        .onChange(of: navigationState.walletTabReselected, model.onWalletTabReselected)
        .bindQuery(
            model.walletQuery,
            model.assetsQuery,
            model.bannersQuery,
            model.fiatValuesQuery,
            model.perpetualBalanceQuery,
            model.collectionsQuery,
        )
        .navigationTitle(Localized.Wallet.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .principal) {
                if !model.isPresentingSearch {
                    WalletBarView(
                        row: model.walletRow,
                        action: model.onSelectWalletBar,
                    )
                    .liquidGlass()
                }
            }
            if !model.isPresentingSearch {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button(action: model.onSelectScanner) {
                        model.scannerImage
                    }
                    .accessibilityIdentifier("scan")
                }
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: model.onToggleSearch) {
                        model.searchImage
                    }
                    .accessibilityIdentifier("search")
                }
            }
        }
        .navigationDestination(keyedBy: Scenes.Asset.self) {
            AssetNavigationView(
                model: viewModelFactory.assetScene(
                    wallet: model.wallet,
                    asset: $0.asset,
                    isPresentingSelectedAssetInput: model.isPresentingSelectedAssetInput,
                    onSelectPerpetuals: { navigationState.wallet.append(Scenes.Perpetuals()) },
                ),
            )
        }
        .navigationDestination(keyedBy: Scenes.NetworkAssets.self) { destination in
            NetworkAssetsScene(
                model: viewModelFactory.networkAssetsScene(
                    wallet: model.wallet,
                    chain: destination.chain,
                    onManageAssets: { model.onSelectManage(chains: [destination.chain]) },
                ),
            )
        }
        .navigationDestination(keyedBy: Scenes.Transaction.self) {
            TransactionNavigationView(model: viewModelFactory.transactionScene(
                transactionId: $0.id,
                wallet: model.wallet,
                onHeaderAction: { action in
                    Task {
                        do {
                            try await navigationRouter.openTransactionAction(action, collectiblePath: navigationState.wallet)
                        } catch {
                            model.isPresentingToastMessage = .error(Localized.Errors.errorOccurred)
                        }
                    }
                },
                onAddContact: { model.isPresentingSheet = .addContact($0) },
                onSelectAddress: { model.isPresentingSheet = .addressDetails($0) },
            ))
        }
        .navigationDestination(keyedBy: Scenes.Collectible.self) {
            CollectibleScene(
                model: viewModelFactory.collectibleScene(
                    wallet: model.wallet,
                    collectible: $0,
                    isPresentingSelectedAssetInput: model.isPresentingSelectedAssetInput,
                    onSelectAddress: { model.isPresentingSheet = .addressDetails($0) },
                ),
            )
        }
        .navigationDestination(keyedBy: Scenes.Collections.self) { _ in
            CollectionsNavigationView(
                model: viewModelFactory.collectionsScene(wallet: model.wallet),
            )
        }
        .navigationDestination(keyedBy: Scenes.Collection.self) { scene in
            CollectionsNavigationView(
                model: viewModelFactory.collectionScene(wallet: model.wallet, collectionId: scene.id),
            )
        }
        .navigationDestination(keyedBy: Scenes.UnverifiedCollections.self) { _ in
            CollectionsNavigationView(
                model: viewModelFactory.unverifiedCollectionsScene(wallet: model.wallet),
            )
        }
        .navigationDestination(keyedBy: Scenes.Chart.self) {
            ChartScene(
                model: viewModelFactory.chartScene(
                    asset: $0.asset,
                    onSetPriceAlert: { presenter.isPresentingPriceAlert.wrappedValue = $0 },
                    onSelectAddress: { model.isPresentingSheet = .addressDetails($0) },
                ),
            )
        }
        .navigationDestination(keyedBy: Scenes.Perpetuals.self) { _ in
            PerpetualsNavigationView(
                model: viewModelFactory.perpetualsScene(
                    wallet: model.wallet,
                    onSelectAmount: { model.isPresentingSheet = .amount($0) },
                    onSelectAssetType: { model.isPresentingSheet = .selectAsset($0, chains: []) },
                    onSelectAsset: navigationRouter.openAsset,
                    onSelectPortfolio: { model.isPresentingSheet = .portfolio(.perpetuals) },
                ),
            )
        }
        .navigationDestination(keyedBy: Scenes.AssetsResults.self) { destination in
            AssetsResultsScene(
                model: viewModelFactory.assetsResultsScene(
                    wallet: model.wallet,
                    request: WalletSearchQuery(
                        walletId: model.wallet.id,
                        searchBy: destination.searchQuery,
                        scope: destination.scope,
                    ),
                    title: destination.title ?? Localized.Assets.title,
                    onSelectAsset: navigationRouter.openAsset,
                ),
            )
        }
        .navigationDestination(keyedBy: Scenes.Perpetual.self) {
            PerpetualNavigationView(
                model: viewModelFactory.perpetualScene(
                    asset: $0.asset,
                    wallet: model.wallet,
                    onTransferData: { model.isPresentingSheet = .transferData($0) },
                    onPerpetualPosition: { model.isPresentingSheet = .amount(AmountInput(type: .perpetual($0), asset: Chain.hyperCore.defaultAsset(type: .perpetual))) },
                ),
                isPresentingSheet: $model.isPresentingSheet,
            )
        }
        .navigationDestination(keyedBy: Scenes.AssetPriceAlert.self) {
            AssetPriceAlertsScene(
                model: viewModelFactory.assetPriceAlertsScene(walletId: model.wallet.id, asset: $0.asset),
            )
        }
        .scanReceiveSheet(isPresented: $model.isPresentingScanner, action: onScan)
        .sheet(item: $model.isPresentingSheet) { sheet in
            Group {
                switch sheet {
                case let .selectAsset(type, chains):
                    SelectAssetNavigationStack(
                        model: viewModelFactory.selectAssetScene(
                            wallet: model.wallet,
                            selectType: type,
                            chains: chains,
                        ),
                    )
                case let .amount(input):
                    AmountNavigationStack(
                        input: input,
                        wallet: model.wallet,
                        onComplete: model.onTransferComplete,
                    )
                case let .infoSheet(type):
                    InfoSheetScene(sheet: type)
                case let .transferData(data):
                    ConfirmTransferNavigationStack(
                        wallet: model.wallet,
                        transferData: data,
                        onComplete: model.onTransferComplete,
                    )
                case .addAsset:
                    AddAssetNavigationStack(wallet: model.wallet)
                case let .portfolio(defaultType):
                    PortfolioScene(model: viewModelFactory.portfolioScene(wallet: model.wallet, defaultType: defaultType))
                case let .addContact(action):
                    AddContactNavigationStack(action: action)
                case let .addressDetails(chainAddress):
                    AddressDetailsDestination(chainAddress: chainAddress)
                case .swap:
                    SwapNavigationStack(wallet: model.wallet, onComplete: model.onTransferComplete)
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
        Task { await navigationRouter.open(code: code) }
    }
}
