// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Style
import SwiftUI

public extension View {
    func contextMenu(_ items: [ContextMenuItemType]) -> some View {
        contextMenuOnOpen { items }
    }

    func contextMenu(_ item: ContextMenuItemType) -> some View {
        contextMenu([item])
    }

    func contextMenuOnOpen(_ items: @escaping () -> [ContextMenuItemType]) -> some View {
        contextMenu {
            ContextMenuItems(items: items)
        }
    }
}

private struct ContextMenuItems: View {
    let items: () -> [ContextMenuItemType]

    var body: some View {
        ForEach(Array(items().enumerated()), id: \.offset) {
            build($0.element)
        }
    }

    @ViewBuilder
    private func build(_ item: ContextMenuItemType) -> some View {
        switch item {
        case let .copy(title, value, expirationTime, onCopied):
            ContextMenuItem(
                title: title ?? Localized.Common.copy,
                systemImage: SystemImage.copy,
            ) {
                Clipboard.copy(value, expirationTime: expirationTime)
                onCopied?(value)
            }
        case let .pin(isPinned, onPin):
            ContextMenuItem(
                title: isPinned ? Localized.Common.unpin : Localized.Common.pin,
                systemImage: isPinned ? SystemImage.unpin : SystemImage.pin,
                action: {
                    onPin?()
                },
            )
        case let .delete(onDelete):
            ContextMenuItem(
                title: Localized.Common.delete,
                systemImage: SystemImage.delete,
                role: .destructive,
                action: {
                    onDelete?()
                },
            )
        case let .url(title, onOpen):
            ContextMenuItem(
                title: title,
                systemImage: SystemImage.globe,
            ) {
                onOpen?()
            }
        case let .custom(title, systemImage, role, action):
            ContextMenuItem(
                title: title,
                systemImage: systemImage,
                role: role,
            ) {
                action?()
            }
        }
    }
}
