// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemAmountService
import struct Gemstone.GemAssetItemRow
import struct Gemstone.GemAutocloseSession
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
@testable import Perpetuals

extension AutocloseType {
    static func mock(
        session: GemAutocloseSession = GemAmountService.mock()
            .newPerpetualSession(action: .open(data: .mock(direction: .long, price: 100, leverage: 10)), format: NumberInput.format())
            .autocloseSession(amount: "1"),
        row: GemAssetItemRow = .mock(),
    ) -> AutocloseType {
        .open(session, row: row, onComplete: { _, _ in })
    }
}
