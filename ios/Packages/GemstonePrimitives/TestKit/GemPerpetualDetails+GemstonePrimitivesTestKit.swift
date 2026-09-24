// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemPerpetualButtonRow
import struct Gemstone.GemPerpetualDetails
import struct Gemstone.GemPerpetualPositionRow
import enum Gemstone.GemPerpetualSection
import struct Gemstone.PerpetualPosition

public extension GemPerpetualDetails {
    static func mock(
        title: String = "BTC",
        sections: [GemPerpetualSection] = [],
        modifyButtons: [GemPerpetualButtonRow] = [],
        position: PerpetualPosition? = nil,
        positionRow: GemPerpetualPositionRow? = nil,
    ) -> GemPerpetualDetails {
        GemPerpetualDetails(
            title: title,
            sections: sections,
            modifyButtons: modifyButtons,
            position: position,
            positionRow: positionRow,
        )
    }
}
