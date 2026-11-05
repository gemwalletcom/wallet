// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Components
import enum Gemstone.GemTransactionHeaderAction
import GemstonePrimitives
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
                        onManageAssets: { present(.wallet(.selectAsset(.manage, chains: [destination.chain]))) },
                    ),
                )
            }
            .navigationDestination(keyedBy: Scenes.Transaction.self, in: wallet.id) {
                TransactionNavigationView(model: viewModelFactory.transactionScene(
                    transactionId: $0.id,
                    wallet: wallet,
                    onHeaderAction: onHeaderAction,
                    onAddContact: { present(.addContact($0)) },
                    onSelectAddress: { present(.addressDetails($0)) },
                ))
            }
            .navigationDestination(keyedBy: Scenes.Collectible.self, in: wallet.id) {
                CollectibleScene(
                    model: viewModelFactory.collectibleScene(
                        wallet: wallet,
                        collectible: $0,
                        isPresentingSelectedAssetInput: presenter.isPresentingAssetInput,
                        onSelectAddress: { present(.addressDetails($0)) },
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
                        onSetPriceAlert: { present(.priceAlert($0)) },
                        onSelectAddress: { present(.addressDetails($0)) },
                    ),
                )
            }
            .navigationDestination(keyedBy: Scenes.Perpetuals.self, in: wallet.id) { _ in
                PerpetualsNavigationView(
                    model: viewModelFactory.perpetualsScene(
                        wallet: wallet,
                        onSelectAmount: { present(.wallet(.amount($0))) },
                        onSelectAssetType: { present(.wallet(.selectAsset($0, chains: []))) },
                        onSelectAsset: navigationRouter.openAsset,
                        onSelectPortfolio: { present(.wallet(.portfolio(.perpetuals))) },
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
                        onTransferData: { present(.wallet(.transferData($0))) },
                        onPerpetualPosition: { present(.wallet(.amount(AmountInput(type: .perpetual($0), asset: Chain.hyperCore.defaultAsset(type: .perpetual))))) },
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
        .toast(message: $isPresentingToastMessage)
    }
}

// MARK: - Actions

extension WalletNavigationStack {
    private func present(_ sheet: NavigationSheetType) {
        presenter.sheet.wrappedValue = sheet
    }

    private func onHeaderAction(_ action: GemTransactionHeaderAction) {
        Task {
            do {
                try await navigationRouter.openTransactionAction(action, collectiblePath: navigationState.wallet)
            } catch {
                isPresentingToastMessage = .error(Localized.Errors.errorOccurred)
            }
        }
    }
}
