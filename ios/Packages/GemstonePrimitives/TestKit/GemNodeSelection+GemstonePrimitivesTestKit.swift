// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemNodeSelection

public extension GemNodeSelection {
    static func mock(url: String) -> GemNodeSelection {
        GemNodeSelection(url: url, host: url, isSelected: false, gemNodeFlag: nil)
    }
}
