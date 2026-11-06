// Copyright (c). Gem Wallet. All rights reserved.

import Components
import PrimitivesComponents
import SwiftUI
import Transactions

struct TransactionsNavigationView: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    @State private var model: TransactionsSceneViewModel

    init(model: TransactionsSceneViewModel) {
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
                    .accessibilityIdentifier("filter")
                }
            }
            .navigationBarTitleDisplayMode(.inline)
            .navigationTitle(model.title)
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
                }
            }
    }
}
