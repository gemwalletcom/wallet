// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import AssetsService
import Components
import Localization
import NFT
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI
import Transactions

struct TransactionsNavigationStack: View {
    @Environment(\.navigationState) private var navigationState
    @Environment(\.assetsEnabler) private var assetsEnabler
    @Environment(\.assetsService) private var assetsService
    @Environment(\.priceAlertService) private var priceAlertService
    @Environment(\.activityService) private var activityService
    @Environment(\.assetSearchService) private var assetSearchService
    @Environment(\.avatarService) private var avatarService
    @Environment(\.navigationPresenter) private var presenter
    @Environment(\.nftService) private var nftService

    @State private var model: TransactionsViewModel

    init(model: TransactionsViewModel) {
        _model = State(wrappedValue: model)
    }

    private var navigationPath: Binding<NavigationPath> {
        navigationState.activity.binding
    }

    var body: some View {
        NavigationStack(path: navigationPath) {
            TransactionsScene(model: model)
                .bindQuery(model.filterModel.query)
                .onChange(
                    of: model.currentWallet,
                    initial: true,
                    model.onChangeWallet,
                )
                .toolbar {
                    ToolbarItem(placement: .navigationBarTrailing) {
                        FilterButton(
                            isActive: model.filterModel.isAnyFilterSpecified,
                            action: model.onSelectFilterButton,
                        )
                    }
                }
                .navigationBarTitleDisplayMode(.inline)
                .navigationTitle(model.title)
                .navigationDestination(for: Scenes.Transaction.self) {
                    TransactionNavigationView(
                        model: TransactionSceneViewModel(
                            transaction: $0.transaction,
                            walletId: model.wallet.id,
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
                            isPresentingSelectedAssetInput: presenter.isPresentingAssetInput,
                        ),
                    )
                }
                .toast(message: $model.isPresentingToastMessage)
                .sheet(item: $model.isPresentingSheet) { type in
                    switch type {
                    case .filter:
                        NavigationStack {
                            TransactionsFilterScene(model: $model.filterModel)
                        }
                        .presentationDetentsForCurrentDeviceSize(expandable: true)
                        .presentationDragIndicator(.visible)
                        .presentationBackground(Colors.grayBackground)
                    case let .selectAsset(selectType):
                        SelectAssetSceneNavigationStack(
                            model: SelectAssetViewModel(
                                wallet: model.wallet,
                                selectType: selectType,
                                searchService: assetSearchService,
                                assetsEnabler: assetsEnabler,
                                priceAlertService: priceAlertService,
                                activityService: activityService,
                            ),
                        )
                    case let .addContact(action):
                        AddContactNavigationView(action: action)
                    }
                }
        }
    }
}

// MARK: - Actions

extension TransactionsNavigationStack {
    private func onSelectTransactionHeaderAction(_ action: TransactionHeaderAction) {
        Task {
            do {
                try await presenter.handleTransactionHeaderAction(
                    action,
                    wallet: model.wallet,
                    navigationState: navigationState,
                    assetsService: assetsService,
                    nftService: nftService,
                    nftDestination: navigationState.activity,
                )
            } catch {
                model.isPresentingToastMessage = .error(Localized.Errors.errorOccured)
            }
        }
    }
}
