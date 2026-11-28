// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemAssetItemRow
import struct Gemstone.GemAutocloseSession
import struct Gemstone.GemPerpetualAmountSession
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
@testable import Perpetuals

extension AutocloseType {
    static func mock(
        session: GemAutocloseSession = GemPerpetualAmountSession.mock(action: .open(data: .mock(direction: .long, price: 100, leverage: 10)), format: NumberInput.format())
            .autocloseSession(amount: "1"),
        row: GemAssetItemRow = .mock(),
    ) -> AutocloseType {
        .open(session, row: row, onComplete: { _, _ in })
    }
}
