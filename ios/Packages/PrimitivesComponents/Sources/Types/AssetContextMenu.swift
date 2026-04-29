// Copyright (c). Gem Wallet. All rights reserved.

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
        [
            .pin(
                isPinned: assetData.metadata.isPinned,
                onPin: onPin,
            ),
            onHide.map { ContextMenuItemType.hide($0) },
            onAddToWallet.map { action in
                !assetData.metadata.isBalanceEnabled ? .custom(
                    title: Localized.Asset.addToWallet,
                    systemImage: SystemImage.plusCircle,
                    action: action,
                ) : nil
            } ?? nil,
            .copy(
                title: Localized.Wallet.copyAddress,
                value: assetData.account.address,
                onCopy: onCopy,
            ),
        ].compactMap(\.self)
    }
}
