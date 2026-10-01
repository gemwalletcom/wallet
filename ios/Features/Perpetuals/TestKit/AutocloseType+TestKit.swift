// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.autocloseOpenSession
import struct Gemstone.GemAssetItemRow
import struct Gemstone.GemAutocloseSession
import GemstonePrimitives
import GemstonePrimitivesTestKit
@testable import Perpetuals

extension AutocloseType {
    static func mock(
        session: GemAutocloseSession = autocloseOpenSession(direction: .long, marketPrice: 100, size: 1, leverage: 10, decimals: 8, provider: .hypercore, format: NumberInput.format()),
        row: GemAssetItemRow = .mock(),
    ) -> AutocloseType {
        .open(session, row: row, onComplete: { _, _ in })
    }
}
