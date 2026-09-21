// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemPerpetualButton
import struct Gemstone.GemPerpetualDetails
import enum Gemstone.GemPerpetualSection
import struct Gemstone.PerpetualPosition

public extension GemPerpetualDetails {
    static func mock(
        title: String = "BTC",
        sections: [GemPerpetualSection] = [],
        modifyButtons: [GemPerpetualButton] = [],
        position: PerpetualPosition? = nil,
    ) -> GemPerpetualDetails {
        GemPerpetualDetails(
            title: title,
            sections: sections,
            modifyButtons: modifyButtons,
            position: position,
        )
    }
}
