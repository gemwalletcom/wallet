// Copyright (c). Gem Wallet. All rights reserved.

import func Gemstone.assetMenuRows
import struct Gemstone.GemAssetMenuInput
import Primitives

public enum AssetContextMenu {
    public static func items(
        for assetData: AssetData,
        onCopy: @escaping (String) -> Void,
        onPin: VoidAction,
        onHide: VoidAction = nil,
        onAddToWallet: VoidAction = nil,
    ) -> [ContextMenuItemType] {
        let rows = assetMenuRows(
            input: GemAssetMenuInput(
                isPinned: assetData.metadata.isPinned,
                isBalanceEnabled: assetData.metadata.isBalanceEnabled,
                address: assetData.account.address,
                offersHide: onHide != nil,
                offersAddToWallet: onAddToWallet != nil,
            ),
        )
        return rows.map { row in
            switch row.action {
            case let .copyAddress(address): .copy(title: row.action.title, value: address, onCopy: onCopy)
            case .pin: .custom(title: row.action.title, systemImage: row.icon.systemImage, action: onPin)
            case .hide: .custom(title: row.action.title, systemImage: row.icon.systemImage, action: onHide)
            case .addToWallet: .custom(title: row.action.title, systemImage: row.icon.systemImage, action: onAddToWallet)
            }
        }
    }
}
