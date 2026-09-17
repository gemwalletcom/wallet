// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemWalletRow

public extension GemWalletRow {
    static func mock(
        id: String = "wallet",
        isPinned: Bool = false,
    ) -> GemWalletRow {
        GemWalletRow(
            id: id,
            name: id,
            subtitle: .address(value: id),
            placeholder: .multicoin,
            showsWatchBadge: false,
            isPinned: isPinned,
            hasAvatar: false,
            imageUrl: nil,
        )
    }
}
