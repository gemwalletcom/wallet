// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemPerpetualMarketSection
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

    @Environment(\.connectionStatus) private var connectionStatus

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
                await model.load()
            }
        }
        .onAppear {
            Task { await model.onAppear() }
        }
        .onDisappear {
            Task { await model.onDisappear() }
        }
        .refreshableTimer(every: connectionStatus.refreshInterval(for: .market)) { source in
            await model.load(source: source)
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

            ForEach(model.marketSectionList, id: \.self) { section in
                marketSection(section)
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

// MARK: - UI Components

extension PerpetualsScene {
    @ViewBuilder
    private func marketSection(_ section: GemPerpetualMarketSection) -> some View {
        switch section {
        case .recents:
            RecentAssetsSectionView(
                model: model.recentModel,
                onSelect: model.onSelectRecent,
            )
        case .positions:
            Section {
                PerpetualPositionsList(
                    positions: model.positions,
                    onSelect: model.onSelectPerpetual,
                )
            } header: {
                Text(section.title)
            }
            .listRowInsets(.assetListRowInsets)
        case .pinned:
            Section {
                PerpetualSectionView(
                    perpetuals: model.sections.pinned,
                    onPin: model.onPinPerpetual,
                    onSelect: model.onSelectPerpetual,
                )
            } header: {
                HStack {
                    model.pinImage
                    Text(section.title)
                }
            }
            .listRowInsets(.assetListRowInsets)
        case .markets:
            Section {
                PerpetualSectionView(
                    perpetuals: model.sections.markets,
                    onPin: model.onPinPerpetual,
                    onSelect: model.onSelectPerpetual,
                )
            } header: {
                Text(section.title)
            }
            .listRowInsets(.assetListRowInsets)
        case .empty:
            EmptyView()
        }
    }
}
