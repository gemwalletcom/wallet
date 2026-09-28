// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct NetworkAssetsScene: View {
    @State private var model: NetworkAssetsSceneViewModel

    public init(model: NetworkAssetsSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        let groups = model.groups
        let assetItems = model.assetItems
        return List {
            if groups.sections.showsPinned {
                Section {
                    assetsList(groups.pinned, itemsModel: assetItems)
                } header: {
                    PinnedSectionHeader()
                }
                .listRowInsets(.assetListRowInsets)
            }

            if groups.sections.showsUnpinned {
                Section {
                    assetsList(groups.unpinned, itemsModel: assetItems)
                }
                .listRowInsets(.assetListRowInsets)
            }

            if groups.sections.showsHidden {
                Section(model.hiddenTitle) {
                    assetsList(groups.hidden, itemsModel: assetItems, onAddToWallet: model.onAddToWallet)
                }
                .listRowInsets(.assetListRowInsets)
            }
        }
        .listStyle(.insetGrouped)
        .listSectionSpacing(.compact)
        .contentMargins([.top], .small, for: .scrollContent)
        .scrollContentBackground(.hidden)
        .background { Colors.insetGroupedListStyle.ignoresSafeArea() }
        .overlay {
            if groups.sections.showsEmpty {
                EmptyContentView(model: model.emptyModel)
            }
        }
        .bindQuery(model.activeQuery, model.hiddenQuery)
        .taskOnce {
            Task { await model.updateBalances() }
        }
        .toast(message: $model.isPresentingToastMessage)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button(action: model.onSelectManageAssets) {
                    model.manageImage
                }
            }
        }
    }

    private func assetsList(_ assets: [AssetData], itemsModel: ListAssetItemsViewModel, onAddToWallet: AssetIdAction = nil) -> some View {
        WalletAssetsList(
            assets: assets,
            itemsModel: itemsModel,
            onHideAsset: model.onHideAsset,
            onPinAsset: model.onPinAsset,
            onAddToWallet: onAddToWallet,
            onCopyAddress: model.onCopyAddress,
            showBalancePrivacy: .constant(false),
        )
    }
}
