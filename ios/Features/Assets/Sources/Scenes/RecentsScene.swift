// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct RecentsScene: View {
    @State private var model: RecentsSceneViewModel

    public init(model: RecentsSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        let state = model.viewState
        NavigationStack {
            List {
                ForEach(state.days, id: \.day) { day in
                    Section {
                        ForEach(day.recents, id: \.asset.id) { recent in
                            let asset = recent.asset.toPrimitives()
                            NavigationCustomLink(
                                with: ListItemView(model: model.listItem(for: asset)),
                            ) {
                                model.onSelect(asset)
                            }
                        }
                    } header: {
                        Text(model.title(for: day))
                            .fontWeight(.semibold)
                    }
                    .listRowInsets(.assetListRowInsets)
                }
            }
            .contentMargins([.top], .extraSmall, for: .scrollContent)
            .listSectionSpacing(.compact)
            .scrollContentBackground(.hidden)
            .background { Colors.sheetInsetGroupedListStyle.ignoresSafeArea() }
            .searchable(
                text: $model.searchQuery,
                placement: .navigationBarDrawer(displayMode: .always),
            )
            .autocorrectionDisabled()
            .overlay {
                if state.sections.empty != nil {
                    EmptyContentView(model: model.emptyModel(state.sections))
                }
            }
            .navigationTitle(model.title)
            .navigationBarTitleDisplayMode(.inline)
            .toolbarContent {
                ToolbarDismissItem(type: .close, placement: .topBarLeading)
                if state.sections.showsClear {
                    ToolbarItemView(placement: .topBarTrailing) {
                        Button(model.clearTitle, action: model.onSelectClear)
                            .bold()
                    }
                }
            }
        }
        .bindQuery(model.query)
    }
}
