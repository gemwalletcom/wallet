// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Components
import struct Gemstone.GemPerpetualMarketItem
import struct Gemstone.GemSearchListRow
import struct Gemstone.GemWalletSearchView
import GemstonePrimitives
import GemstoneServices
import NFT
import Perpetuals
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct WalletSearchScene: View {
    @State private var model: WalletSearchSceneViewModel

    public init(model: WalletSearchSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        let view = model.view
        return SearchableWrapper(
            content: { content(view) },
            isSearching: $model.isSearching,
            dismissSearch: $model.dismissSearch,
        )
        .searchStateOverlay(model.searchState(view), background: Colors.sheetInsetGroupedListStyle)
        .bindQuery(model.searchQuery, model.recentModel.query)
        .searchable(
            text: $model.searchableQuery,
            isPresented: $model.isSearchPresented,
            placement: .navigationBarDrawer(displayMode: .always),
        )
        .autocorrectionDisabled(true)
        .debounce(
            value: $model.searchableQuery.wrappedValue,
            interval: GemConstants.searchDebounce,
            action: model.onSearch(query:),
        )
        .onChange(of: model.searchableQuery, model.onChangeSearchQuery)
        .onChange(of: model.isSearchPresented, model.onChangeSearchPresented)
        .onAppear {
            model.onAppear()
        }
        .taskOnce {
            model.load()
        }
        .toast(message: $model.isPresentingToastMessage)
        .recentAssetsSheet(model: model.recentModel, onSelect: model.onSelectRecent)
    }

    private func content(_ view: GemWalletSearchView) -> some View {
        let state = view.state
        return List {
            if state.showsRecents {
                RecentAssetsSectionView(
                    model: model.recentModel,
                    onSelect: model.onSelectRecent,
                )
            }

            if state.showsPinned {
                Section(
                    content: {
                        perpetualItems(for: view.pinnedPerpetuals)
                        assetItems(for: model.assets(view.pinnedAssetIds))
                    },
                    header: { PinnedSectionHeader() },
                )
                .listRowInsets(.assetListRowInsets)
            }

            if state.showsLists {
                Section(
                    content: { listItems(for: view.lists) },
                    header: { SectionHeaderView(title: model.listsTitle) },
                )
                .listRowInsets(.assetListRowInsets)
            }

            if state.showsPerpetuals {
                Section(
                    content: { perpetualItems(for: view.perpetuals) },
                    header: {
                        if view.hasMorePerpetuals {
                            HeaderNavigationLinkView(title: model.perpetualsTitle, destination: Scenes.Perpetuals())
                        } else {
                            SectionHeaderView(title: model.perpetualsTitle)
                        }
                    },
                )
                .listRowInsets(.assetListRowInsets)
            }

            if state.showsNfts {
                Section(
                    content: { CollectionsPreviewView(entries: view.nfts) },
                    header: {
                        if view.hasMoreNfts {
                            HeaderNavigationLinkView(title: model.collectionsTitle, destination: Scenes.Collections())
                        } else {
                            SectionHeaderView(title: model.collectionsTitle)
                        }
                    },
                )
                .listRowInsets(.assetListRowInsets)
            }

            if state.showsAssets {
                Section(
                    content: { assetItems(for: model.assets(view.assetIds)) },
                    header: {
                        if view.hasMoreAssets {
                            HeaderNavigationLinkView(
                                title: model.assetsTitle,
                                destination: model.assetsResultsDestination,
                            )
                        } else {
                            SectionHeaderView(title: model.assetsTitle)
                        }
                    },
                )
                .listRowInsets(.assetListRowInsets)
            }
        }
        .contentMargins([.top], .extraSmall, for: .scrollContent)
        .listSectionSpacing(.compact)
    }

    private func assetItems(for items: [AssetData]) -> some View {
        AssetItemsView(
            items: items,
            itemsModel: model.assetItems,
            contextMenuItems: model.contextMenuItems,
            onSelect: model.onSelectAsset,
        )
    }

    private func listItems(for rows: [GemSearchListRow]) -> some View {
        ForEach(rows, id: \.list.id) { row in
            NavigationLink(value: model.listDestination(for: row)) {
                ListItemView(model: row.listItem)
            }
        }
    }

    private func perpetualItems(for items: [GemPerpetualMarketItem]) -> some View {
        PerpetualSectionView(items: items, onPin: model.onSelectPinPerpetual, onSelect: model.onSelectAsset)
    }
}
