// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstoneServices
import Preferences
import Primitives
import PrimitivesComponents
import Recents
import Store
import Style
import SwiftUI

struct PerpetualsScene: View {
    @Bindable private var model: PerpetualsSceneViewModel

    init(model: PerpetualsSceneViewModel) {
        self.model = model
    }

    var body: some View {
        SearchableWrapper(
            content: { list },
            isSearching: $model.isSearching,
            dismissSearch: .constant(false),
        )
        .searchable(
            text: $model.searchQuery,
            isPresented: $model.isSearchPresented,
            placement: .navigationBarDrawer(displayMode: .automatic),
        )
        .onChange(of: model.searchQuery, model.onSearchQueryChange)
        .onChange(of: model.isSearchPresented, model.onSearchPresentedChange)
        .navigationTitle(model.navigationTitle)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                Button(action: model.onSelectSearchButton) {
                    model.searchImage
                }
            }
        }
        .taskOnce {
            Task {
                await model.fetch()
            }
        }
        .onAppear {
            Task { await model.onAppear() }
        }
        .onDisappear {
            Task { await model.onDisappear() }
        }
        .refreshableTimer(every: .minutes(1)) {
            await model.fetch()
        }
        .listSectionSpacing(.compact)
        .recentAssetsSheet(model: model.recentModel, onSelect: model.onSelectRecent)
    }

    var list: some View {
        List {
            if !model.isSearching {
                Section {} header: {
                    ValueHeaderView(
                        model: model.headerViewModel,
                        isPrivacyEnabled: .constant(false),
                        titleActionType: .action(model.onSelectBalance),
                        onHeaderAction: model.onSelectHeaderAction,
                        onInfoAction: .none,
                    )
                    .padding(.top, Spacing.small)
                }
                .cleanListRow()
            }

            if model.showRecents {
                RecentAssetsSectionView(
                    model: model.recentModel,
                    onSelect: model.onSelectRecent,
                )
            }

            if model.showPositions {
                Section {
                    PerpetualPositionsList(
                        positions: model.positions,
                        onSelect: model.onSelectPerpetual,
                    )
                } header: {
                    Text(model.positionsSectionTitle)
                }
                .listRowInsets(.assetListRowInsets)
            }

            if model.showPinned {
                Section {
                    PerpetualSectionView(
                        perpetuals: model.sections.pinned,
                        onPin: model.onPinPerpetual,
                        onSelect: model.onSelectPerpetual,
                    )
                } header: {
                    HStack {
                        model.pinImage
                        Text(model.pinnedSectionTitle)
                    }
                }
                .listRowInsets(.assetListRowInsets)
            }

            if model.showMarkets {
                Section {
                    PerpetualSectionView(
                        perpetuals: model.sections.markets,
                        onPin: model.onPinPerpetual,
                        onSelect: model.onSelectPerpetual,
                    )
                } header: {
                    Text(model.marketsSectionTitle)
                }
                .listRowInsets(.assetListRowInsets)
            }
        }
        .if(!model.isSearching) {
            $0.contentMargins([.top], .space12, for: .scrollContent)
        }
        .overlay {
            if model.showSearchEmptyState {
                EmptyContentView(model: model.emptyContentModel)
            }
        }
    }
}
