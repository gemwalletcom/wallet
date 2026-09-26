// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemValidatorRow
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Stake
import Testing

struct GemValidatorRowTests {
    @Test func aprText() {
        #expect(GemValidatorRow.mock(apr: .apr(value: .mock(value: 2.15, unit: .percent, display: .number(precision: .fraction(min: 2, max: 2)), notation: .plain, tone: .plain, rounding: .toNearest))).listItem.subtitle == "APR 2.15%")
        #expect(GemValidatorRow.mock(apr: .apr(value: nil)).listItem.subtitle == "APR ")
    }
}
