// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemWalletSearchState
import struct Gemstone.GemWalletSearchView
import GemstonePrimitives
import GemstoneServices
import Localization
import NFT
import Perpetuals
import Primitives
import PrimitivesComponents
import Recents
import Store
import Style
import SwiftUI

public struct WalletSearchScene: View {
    @State private var model: WalletSearchSceneViewModel

    public init(model: WalletSearchSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        let search = model.derived
        return SearchableWrapper(
            content: { content(search) },
            isSearching: $model.isSearching,
            dismissSearch: $model.dismissSearch,
        )
        .searchStateOverlay(model.searchState(search), background: Colors.sheetInsetGroupedListStyle)
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

    private func content(_ search: WalletSearchDerived) -> some View {
        let state = search.view.state
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
                        if state.showsPinnedPerpetuals {
                            perpetualItems(for: search.sections.pinnedPerpetuals)
                        }
                        assetItems(for: search.sections.pinnedAssets)
                    },
                    header: { PinnedSectionHeader() },
                )
                .listRowInsets(.assetListRowInsets)
            }

            if state.showsLists {
                Section(
                    content: { listItems(for: search.sections.lists) },
                    header: { SectionHeaderView(title: model.listsTitle) },
                )
                .listRowInsets(.assetListRowInsets)
            }

            if state.showsPerpetuals {
                Section(
                    content: { perpetualItems(for: search.previewPerpetuals) },
                    header: {
                        if search.view.hasMorePerpetuals {
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
                    content: { CollectionsPreviewView(content: search.collectionsContent) },
                    header: {
                        if search.view.hasMoreNfts {
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
                    content: { assetItems(for: search.previewAssets) },
                    header: {
                        if search.view.hasMoreAssets {
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

    private func listItems(for lists: [AssetList]) -> some View {
        ForEach(lists) { list in
            NavigationLink(value: model.listDestination(for: list)) {
                ListItemView(model: model.listItem(for: list))
            }
        }
    }

    private func perpetualItems(for items: [PerpetualData]) -> some View {
        PerpetualSectionView(perpetuals: items, onPin: model.onSelectPinPerpetual, onSelect: model.onSelectAsset)
    }
}
