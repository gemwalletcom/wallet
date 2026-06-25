// Copyright (c). Gem Wallet. All rights reserved.

import Components
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
        List {
            if model.showPinned {
                Section(
                    content: { assetItems(for: model.sections.pinnedAssets) },
                    header: { PinnedSectionHeader() },
                )
                .listRowInsets(.assetListRowInsets)
            }

            if model.showAssets {
                Section {
                    assetItems(for: model.sections.assets)
                }
                .listRowInsets(.assetListRowInsets)
            }

            if model.showPerpetuals {
                Section(
                    content: {
                        PerpetualItemsView(
                            items: model.perpetuals,
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
        .searchStateOverlay(isLoading: model.showLoading, isEmpty: model.showEmpty, empty: .search(type: .assets))
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .bindQuery(model.searchQuery)
        .taskOnce {
            model.fetch()
        }
        .toast(message: $model.isPresentingToastMessage)
    }

    private func assetItems(for items: [AssetData]) -> some View {
        AssetItemsView(
            items: items,
            currencyCode: model.currencyCode,
            contextMenuItems: model.contextMenuItems,
            onSelect: { model.onSelectAsset($0) },
        )
    }
}
