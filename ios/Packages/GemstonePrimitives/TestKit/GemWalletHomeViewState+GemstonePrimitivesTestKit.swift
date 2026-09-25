// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemFormattedNumber
import enum Gemstone.GemHeaderActions
import enum Gemstone.GemLocalizedText
import enum Gemstone.GemValueTone
import struct Gemstone.GemWalletHomeViewState

public extension GemWalletHomeViewState {
    static func mock(
        total: GemFormattedNumber = .mock(),
        pnl: GemLocalizedText? = nil,
        pnlTone: GemValueTone = .plain,
        headerActions: GemHeaderActions = .buttons(buttons: []),
    ) -> GemWalletHomeViewState {
        GemWalletHomeViewState(
            total: total,
            pnl: pnl,
            pnlTone: pnlTone,
            headerActions: headerActions,
            showCollections: false,
            showsPerpetuals: false,
            banner: nil,
        )
    }
}
