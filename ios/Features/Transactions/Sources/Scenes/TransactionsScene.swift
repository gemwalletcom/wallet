// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct TransactionsScene: View {
    private var model: TransactionsSceneViewModel

    public init(model: TransactionsSceneViewModel) {
        self.model = model
    }

    @Environment(\.connectionStatus) private var connectionStatus

    public var body: some View {
        let phase = model.phase
        VStack {
            List {
                if case let .error(error) = phase {
                    Section {
                        ListItemErrorView(errorTitle: Localized.Errors.errorOccurred, error: error)
                    }
                }
                TransactionsList(sections: model.sections)
                    .listRowInsets(.assetListRowInsets)
            }
            .listSectionSpacing(.compact)
            .scrollContentBackground(.hidden)
            .refreshableTimer(every: connectionStatus.refreshInterval(for: .wallet)) { _ in
                await model.load()
            }
        }
        .background { Colors.insetGroupedListStyle.ignoresSafeArea() }
        .overlay {
            if case let .empty(state) = phase {
                EmptyContentView(model: model.emptyContentModel(state))
                    .padding(.horizontal, .medium)
            }
        }
        .task { await model.load() }
    }
}
