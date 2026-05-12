// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import AssetsService
import Components
import NFT
import Primitives
import Store
import Style
import SwiftUI
import Transactions

struct TransactionsNavigationStack: View {
    @Environment(\.navigationState) private var navigationState
    @Environment(\.assetsEnabler) private var assetsEnabler
    @Environment(\.priceAlertService) private var priceAlertService
    @Environment(\.activityService) private var activityService
    @Environment(\.assetSearchService) private var assetSearchService
    @Environment(\.avatarService) private var avatarService
    @Environment(\.navigationPresenter) private var presenter
    @Environment(\.nftService) private var nftService
    @Environment(\.openURL) private var openURL

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
                    }
                }
        }
    }
}

// MARK: - Actions

extension TransactionsNavigationStack {
    private func onSelectTransactionHeaderAction(_ action: TransactionHeaderAction) {
        switch action {
        case let .url(url):
            openURL(url)
        case let .nft(assetId):
            Task {
                do {
                    let assetData = try await nftService.assetData(assetId: assetId)
                    navigationState.activity.append(Scenes.Collectible(assetData: assetData))
                } catch {
                    debugLog("Open NFT details error: \(error)")
                }
            }
        }
    }
}
