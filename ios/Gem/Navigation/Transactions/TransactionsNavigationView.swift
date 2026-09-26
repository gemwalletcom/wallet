// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstonePrimitives
import Localization
import NFT
import Primitives
import PrimitivesComponents
import Store
import SwiftUI
import Transactions

struct TransactionsNavigationView: View {
    @Environment(\.navigationState) private var navigationState
    @Environment(\.viewModelFactory) private var viewModelFactory
    @Environment(\.navigationPresenter) private var presenter

    @State private var model: TransactionsViewModel

    init(model: TransactionsViewModel) {
        _model = State(wrappedValue: model)
    }

    var body: some View {
        TransactionsScene(model: model)
            .bindQuery(model.filterModel.query)
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
                if let sceneModel = viewModelFactory.transactionScene(
                    transactionId: $0.id,
                    wallet: model.wallet,
                    onHeaderAction: { action in
                        Task {
                            do {
                                try await presenter.openTransactionHeaderAction(
                                    action,
                                    wallet: model.wallet,
                                    navigationState: navigationState,
                                    nftDestination: navigationState.activity,
                                )
                            } catch {
                                model.isPresentingToastMessage = .error(Localized.Errors.errorOccurred)
                            }
                        }
                    },
                    onAddContact: { model.isPresentingSheet = .addContact($0) },
                    onSelectAddress: { model.isPresentingSheet = .addressDetails($0) },
                ) {
                    TransactionNavigationView(model: sceneModel)
                }
            }
            .navigationDestination(for: Scenes.Collectible.self) {
                CollectibleScene(
                    model: viewModelFactory.collectibleScene(
                        wallet: model.wallet,
                        assetData: $0.assetData,
                        isPresentingSelectedAssetInput: presenter.isPresentingAssetInput,
                        onSelectAddress: { model.isPresentingSheet = .addressDetails($0) },
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
                    .sheetPresentation(.forCurrentDeviceSize(expandable: true), dragIndicator: .visible)
                case let .selectAsset(selectType):
                    SelectAssetNavigationStack(
                        model: viewModelFactory.selectAssetScene(
                            wallet: model.wallet,
                            selectType: selectType,
                        ),
                    )
                case let .addContact(action):
                    AddContactNavigationView(action: action)
                case let .addressDetails(chainAddress):
                    AddressDetailsDestination(chainAddress: chainAddress)
                }
            }
    }
}
