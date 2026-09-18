// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemNodeSyncState
import Style

extension GemNodeSyncState {
    var symbol: String {
        switch self {
        case .inSync: Emoji.checkmark
        case .outOfSync: Emoji.reject
        }
    }
}
