// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemWalletSearchState
import Perpetuals
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct AssetsResultsScene: View {
    @State private var model: AssetsResultsSceneViewModel

    public init(model: AssetsResultsSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        let state = model.state
        let sections = model.sections
        return List {
            if state.showsPinned {
                Section(
                    content: { assetItems(for: sections.pinnedAssets) },
                    header: { PinnedSectionHeader() },
                )
                .listRowInsets(.assetListRowInsets)
            }

            if state.showsAssets {
                Section {
                    assetItems(for: sections.assets)
                }
                .listRowInsets(.assetListRowInsets)
            }

            if state.showsPerpetuals {
                Section(
                    content: {
                        PerpetualSectionView(
                            perpetuals: model.perpetuals,
                            onPin: model.onSelectPinPerpetual,
                            onSelect: { model.onSelectAsset($0) },
                        )
                    },
                    header: { SectionHeaderView(title: model.perpetualsTitle) },
                )
            }
        }
        .listSectionSpacing(.compact)
        .refreshable {
            await model.refresh()
        }
        .searchStateOverlay(model.searchState(state), background: Colors.insetGroupedListStyle)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .bindQuery(model.searchQuery)
        .taskOnce {
            model.load()
        }
        .toast(message: $model.isPresentingToastMessage)
    }

    private func assetItems(for items: [AssetData]) -> some View {
        AssetItemsView(
            items: items,
            itemsModel: model.assetItems,
            contextMenuItems: model.contextMenuItems,
            onSelect: { model.onSelectAsset($0) },
        )
    }
}
