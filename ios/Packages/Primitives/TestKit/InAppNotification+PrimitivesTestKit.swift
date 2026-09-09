// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension CoreListItem {
    static func mock(
        id: String = "mock_id",
        title: String = "Mock",
        subtitle: String? = nil,
        value: String? = nil,
        subvalue: String? = nil,
        icon: CoreListItemIcon? = nil,
        badge: CoreListItemBadge? = nil,
        url: String? = nil,
    ) -> CoreListItem {
        CoreListItem(
            id: id,
            title: title,
            subtitle: subtitle,
            value: value,
            subvalue: subvalue,
            icon: icon,
            badge: badge,
            url: url,
        )
    }
}

public extension InAppNotification {
    static func mock(
        walletId: WalletId = .mock(),
        readAt: Date? = nil,
        createdAt: Date = .now,
        item: CoreListItem = .mock(),
    ) -> InAppNotification {
        InAppNotification(
            walletId: walletId,
            readAt: readAt,
            createdAt: createdAt,
            item: item,
        )
    }
}
