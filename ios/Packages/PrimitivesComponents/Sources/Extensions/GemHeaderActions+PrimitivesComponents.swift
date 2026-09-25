// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemHeaderActions

public extension GemHeaderActions {
    var isWatchOnly: Bool {
        self == .watchOnly
    }

    var headerButtons: [HeaderButton] {
        switch self {
        case .watchOnly: []
        case let .buttons(buttons): buttons.map { HeaderButton(type: $0.kind, isEnabled: $0.isEnabled) }
        }
    }
}
