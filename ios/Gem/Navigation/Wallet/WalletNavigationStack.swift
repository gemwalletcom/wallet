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

struct WalletNavigationStack: View {
    @Environment(\.navigationRouter) private var navigationRouter
    @Environment(\.navigationState) private var navigationState
    @Environment(\.navigationPresenter) private var presenter
    @Environment(\.viewModelFactory) private var viewModelFactory

    let wallet: Wallet

    @State private var isPresentingToastMessage: ToastMessage?

    var body: some View {
        NavigationStack(path: navigationState.wallet.binding) {
            WalletNavigationView(
                model: viewModelFactory.walletScene(
                    wallet: wallet,
                    isPresentingSelectedAssetInput: presenter.isPresentingAssetInput,
                    isPresentingWallets: presenter.isPresentingWallets,
                    isPresentingSheet: presenter.isPresentingWalletSheet,
                ),
            )
            .id(wallet.id)
            .navigationDestination(keyedBy: Scenes.Asset.self, in: wallet.id) {
                AssetNavigationView(
                    model: viewModelFactory.assetScene(
                        wallet: wallet,
                        asset: $0.asset,
                        isPresentingSelectedAssetInput: presenter.isPresentingAssetInput,
                        onSelectPerpetuals: { navigationState.wallet.append(Scenes.Perpetuals()) },
                    ),
                )
            }
            .navigationDestination(keyedBy: Scenes.NetworkAssets.self, in: wallet.id) { destination in
                NetworkAssetsScene(
                    model: viewModelFactory.networkAssetsScene(
                        wallet: wallet,
                        chain: destination.chain,
                        onManageAssets: { isPresentingSheet = .selectAsset(.manage, chains: [destination.chain]) },
                    ),
                )
            }
            .navigationDestination(keyedBy: Scenes.Transaction.self, in: wallet.id) {
                TransactionNavigationView(model: viewModelFactory.transactionScene(
                    transactionId: $0.id,
                    wallet: wallet,
                    onHeaderAction: { action in
                        Task {
                            do {
                                try await navigationRouter.openTransactionAction(action, collectiblePath: navigationState.wallet)
                            } catch {
                                isPresentingToastMessage = .error(Localized.Errors.errorOccurred)
                            }
                        }
                    },
                    onAddContact: { isPresentingSheet = .addContact($0) },
                    onSelectAddress: { isPresentingSheet = .addressDetails($0) },
                ))
            }
            .navigationDestination(keyedBy: Scenes.Collectible.self, in: wallet.id) {
                CollectibleScene(
                    model: viewModelFactory.collectibleScene(
                        wallet: wallet,
                        collectible: $0,
                        isPresentingSelectedAssetInput: presenter.isPresentingAssetInput,
                        onSelectAddress: { isPresentingSheet = .addressDetails($0) },
                    ),
                )
            }
            .navigationDestination(keyedBy: Scenes.Collections.self, in: wallet.id) { _ in
                CollectionsNavigationView(
                    model: viewModelFactory.collectionsScene(wallet: wallet),
                )
            }
            .navigationDestination(keyedBy: Scenes.Collection.self, in: wallet.id) { scene in
                CollectionsNavigationView(
                    model: viewModelFactory.collectionScene(wallet: wallet, collectionId: scene.id),
                )
            }
            .navigationDestination(keyedBy: Scenes.UnverifiedCollections.self, in: wallet.id) { _ in
                CollectionsNavigationView(
                    model: viewModelFactory.unverifiedCollectionsScene(wallet: wallet),
                )
            }
            .navigationDestination(keyedBy: Scenes.Chart.self, in: wallet.id) {
                ChartScene(
                    model: viewModelFactory.chartScene(
                        asset: $0.asset,
                        onSetPriceAlert: { presenter.isPresentingPriceAlert.wrappedValue = $0 },
                        onSelectAddress: { isPresentingSheet = .addressDetails($0) },
                    ),
                )
            }
            .navigationDestination(keyedBy: Scenes.Perpetuals.self, in: wallet.id) { _ in
                PerpetualsNavigationView(
                    model: viewModelFactory.perpetualsScene(
                        wallet: wallet,
                        onSelectAmount: { isPresentingSheet = .amount($0) },
                        onSelectAssetType: { isPresentingSheet = .selectAsset($0, chains: []) },
                        onSelectAsset: navigationRouter.openAsset,
                        onSelectPortfolio: { isPresentingSheet = .portfolio(.perpetuals) },
                    ),
                )
            }
            .navigationDestination(keyedBy: Scenes.AssetsResults.self, in: wallet.id) { destination in
                AssetsResultsScene(
                    model: viewModelFactory.assetsResultsScene(
                        wallet: wallet,
                        request: WalletSearchQuery(
                            walletId: wallet.id,
                            searchBy: destination.searchQuery,
                            scope: destination.scope,
                        ),
                        title: destination.title ?? Localized.Assets.title,
                        onSelectAsset: navigationRouter.openAsset,
                    ),
                )
            }
            .navigationDestination(keyedBy: Scenes.Perpetual.self, in: wallet.id) {
                PerpetualNavigationView(
                    model: viewModelFactory.perpetualScene(
                        asset: $0.asset,
                        wallet: wallet,
                        onTransferData: { isPresentingSheet = .transferData($0) },
                        onPerpetualPosition: { isPresentingSheet = .amount(AmountInput(type: .perpetual($0), asset: Chain.hyperCore.defaultAsset(type: .perpetual))) },
                    ),
                    isPresentingSheet: presenter.isPresentingWalletSheet,
                )
            }
            .navigationDestination(keyedBy: Scenes.AssetPriceAlert.self, in: wallet.id) {
                AssetPriceAlertsScene(
                    model: viewModelFactory.assetPriceAlertsScene(walletId: wallet.id, asset: $0.asset),
                )
            }
        }
        .sheet(item: presenter.isPresentingWalletSheet) { sheet in
            Group {
                switch sheet {
                case let .selectAsset(type, chains):
                    SelectAssetNavigationStack(
                        model: viewModelFactory.selectAssetScene(
                            wallet: wallet,
                            selectType: type,
                            chains: chains,
                        ),
                    )
                case let .amount(input):
                    AmountNavigationStack(
                        input: input,
                        wallet: wallet,
                        onComplete: onTransferComplete,
                    )
                case let .infoSheet(type):
                    InfoSheetScene(sheet: type)
                case let .transferData(data):
                    ConfirmTransferNavigationStack(
                        wallet: wallet,
                        transferData: data,
                        onComplete: onTransferComplete,
                    )
                case .addAsset:
                    AddAssetNavigationStack(wallet: wallet)
                case let .portfolio(defaultType):
                    PortfolioScene(model: viewModelFactory.portfolioScene(wallet: wallet, defaultType: defaultType))
                case let .addContact(action):
                    AddContactNavigationStack(action: action)
                case let .addressDetails(chainAddress):
                    AddressDetailsDestination(chainAddress: chainAddress)
                case .swap:
                    SwapNavigationStack(wallet: wallet, onComplete: onTransferComplete)
                }
            }
            .id(sheet.id)
        }
        .toast(message: $isPresentingToastMessage)
    }
}

// MARK: - Actions

extension WalletNavigationStack {
    private var isPresentingSheet: WalletSheetType? {
        get { presenter.isPresentingWalletSheet.wrappedValue }
        nonmutating set { presenter.isPresentingWalletSheet.wrappedValue = newValue }
    }

    private func onTransferComplete() {
        isPresentingSheet = nil
    }
}
