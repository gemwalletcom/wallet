// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemHeaderButtonKind

public extension GemHeaderButtonKind {
    var headerButtonType: HeaderButtonType {
        switch self {
        case .send: .send
        case .receive: .receive
        case .buy: .buy
        case .swap: .swap
        }
    }
}
