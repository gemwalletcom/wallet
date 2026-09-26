// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct FiatTransactionsScene: View {
    @State private var model: FiatTransactionsSceneViewModel

    public init(model: FiatTransactionsSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        let sections = model.sections
        let loadError = model.loadError
        return List {
            if let error = loadError {
                Section {
                    ListItemErrorView(errorTitle: Localized.Errors.errorOccurred, error: error)
                }
            }
            ForEach(sections) { section in
                Section {
                    ForEach(section.values) { row in
                        if let url = row.detailsURL {
                            SafariNavigationLink(url: url) {
                                ListItemView(model: row.listItemModel)
                            }
                        } else {
                            NavigationCustomLink(
                                with: ListItemView(model: row.listItemModel),
                                action: {},
                            )
                        }
                    }
                } header: {
                    section.title.map { Text($0) }
                }
            }
            .listRowInsets(.assetListRowInsets)
        }
        .overlay {
            if sections.isEmpty, loadError == nil {
                EmptyContentView(model: model.emptyContentModel)
                    .padding(.horizontal, .medium)
            }
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .bindQuery(model.query)
        .refreshable { await model.load() }
        .navigationTitle(model.title)
        .task { await model.load() }
    }
}
