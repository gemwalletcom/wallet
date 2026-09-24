// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct WalletAssetsList: View {
    let assets: [AssetData]
    let itemsModel: ListAssetItemsViewModel
    let onHideAsset: AssetIdAction
    let onPinAsset: AssetBoolAction
    let onAddToWallet: AssetIdAction
    let onCopyAddress: ((String) -> Void)?

    @Binding var showBalancePrivacy: Bool

    init(
        assets: [AssetData],
        itemsModel: ListAssetItemsViewModel,
        onHideAsset: AssetIdAction,
        onPinAsset: AssetBoolAction,
        onAddToWallet: AssetIdAction = nil,
        onCopyAddress: ((String) -> Void)? = nil,
        showBalancePrivacy: Binding<Bool>,
    ) {
        self.assets = assets
        self.itemsModel = itemsModel
        self.onHideAsset = onHideAsset
        self.onPinAsset = onPinAsset
        self.onAddToWallet = onAddToWallet
        self.onCopyAddress = onCopyAddress
        _showBalancePrivacy = showBalancePrivacy
    }

    var body: some View {
        ForEach(assets) { asset in
            NavigationLink(value: Scenes.Asset(asset: asset.asset)) {
                ListAssetItemView(model: itemsModel.item(asset, showBalancePrivacy: $showBalancePrivacy))
                    .contextMenu(
                        AssetContextMenu.items(
                            for: asset,
                            onCopy: { onCopyAddress?(itemsModel.copyMessage(chain: asset.asset.chain, address: $0)) },
                            onPin: { onPinAsset?(asset.asset, !asset.metadata.isPinned) },
                            onHide: asset.metadata.isBalanceEnabled ? { onHideAsset?(asset.asset.id) } : nil,
                            onAddToWallet: onAddToWallet.map { action in { action(asset.asset.id) } },
                        ),
                    )
                    .swipeActions(edge: .trailing) {
                        Button(role: .destructive) {
                            onHideAsset?(asset.asset.id)
                        } label: {
                            Label(
                                Localized.Common.hide,
                                systemImage: SystemImage.hide,
                            )
                        }
                        .tint(Colors.red)
                    }
                    .swipeActions(edge: .leading) {
                        Button(role: .destructive) {
                            onPinAsset?(asset.asset, !asset.metadata.isPinned)
                        } label: {
                            Label(
                                asset.metadata.isPinned ? Localized.Common.unpin : Localized.Common.pin,
                                systemImage: asset.metadata.isPinned ? SystemImage.unpin : SystemImage.pin,
                            )
                        }
                        .tint(Colors.green)
                    }
            }
        }
    }
}
