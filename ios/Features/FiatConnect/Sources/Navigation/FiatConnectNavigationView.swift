// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import Store
import Style
import SwiftUI

public struct FiatConnectNavigationView: View {
    @State private var model: FiatSceneViewModel

    public init(model: FiatSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        FiatScene(
            model: model,
        )
        .onChangeBindQuery(model.assetQuery, action: model.onAssetDataChange)
        .ifElse(
            model.showFiatTypePicker,
            ifContent: {
                $0.toolbar {
                    FiatTypeToolbar(selectedType: $model.type)
                }
            },
            elseContent: {
                $0.navigationTitle(model.title)
            },
        )
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                NavigationLink(value: Scenes.FiatTransactions()) {
                    Images.Tabs.activity
                }
            }
        }
        .navigationDestination(for: Scenes.FiatTransactions.self) { _ in
            FiatTransactionsScene(
                model: FiatTransactionsViewModel(
                    walletId: model.walletId,
                    service: model.fiatService,
                ),
            )
        }
        .sheet(isPresented: $model.isPresentingFiatProvider) {
            SelectableListNavigationStack(
                model: model.fiatProviderViewModel,
                onFinishSelection: model.onSelectQuotes,
                listContent: { SimpleListItemView(model: $0) },
            )
        }
    }
}
