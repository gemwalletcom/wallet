// Copyright (c). Gem Wallet. All rights reserved.

import func Gemstone.assetMenuActions
import enum Gemstone.GemAssetMenuAction
import struct Gemstone.GemAssetMenuInput
import Localization
import Primitives
import Style

public enum AssetContextMenu {
    public static func items(
        for assetData: AssetData,
        onCopy: @escaping (String) -> Void,
        onPin: VoidAction,
        onHide: VoidAction = nil,
        onAddToWallet: VoidAction = nil,
    ) -> [ContextMenuItemType] {
        let actions = assetMenuActions(
            input: GemAssetMenuInput(
                isPinned: assetData.metadata.isPinned,
                isBalanceEnabled: assetData.metadata.isBalanceEnabled,
                address: assetData.account.address,
                offersHide: onHide != nil,
                offersAddToWallet: onAddToWallet != nil,
            ),
        )
        return actions.compactMap { action in
            switch action {
            case let .pin(isPinned):
                .pin(isPinned: isPinned, onPin: onPin)
            case .hide:
                onHide.map { ContextMenuItemType.hide($0) }
            case .addToWallet:
                onAddToWallet.map {
                    ContextMenuItemType.custom(
                        title: action.title ?? "",
                        systemImage: SystemImage.plusCircle,
                        action: $0,
                    )
                }
            case let .copyAddress(address):
                .copy(title: action.title, value: address, onCopy: onCopy)
            }
        }
    }
}
