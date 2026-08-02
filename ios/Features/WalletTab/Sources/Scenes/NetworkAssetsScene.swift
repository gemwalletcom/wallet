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
        List {
            if model.showPinned {
                Section {
                    assetsList(model.pinned)
                } header: {
                    PinnedSectionHeader()
                }
                .listRowInsets(.assetListRowInsets)
            }

            if model.showUnpinned {
                Section {
                    assetsList(model.unpinned)
                }
                .listRowInsets(.assetListRowInsets)
            }

            if model.showHidden {
                Section(model.hiddenTitle) {
                    assetsList(model.hidden, onAddToWallet: model.onAddToWallet)
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
            if model.showEmpty {
                EmptyContentView(model: model.emptyModel)
            }
        }
        .bindQuery(model.activeQuery, model.hiddenQuery)
        .task(id: model.assetIds) {
            await model.updateBalances()
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

    private func assetsList(_ assets: [AssetData], onAddToWallet: AssetIdAction = nil) -> some View {
        WalletAssetsList(
            assets: assets,
            currencyCode: model.currencyCode,
            onHideAsset: model.onHideAsset,
            onPinAsset: model.onPinAsset,
            onAddToWallet: onAddToWallet,
            onCopyAddress: model.onCopyAddress,
            showBalancePrivacy: .constant(model.hideBalance),
        )
    }
}
