// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Stake
import Testing

struct ValidatorViewModelTests {
    @Test func aprText() {
        #expect(ValidatorViewModel(row: .mock(apr: .apr(value: .mock(value: 2.15, unit: .percent, display: .number(precision: .fraction(min: 2, max: 2)), notation: .plain, tone: .plain, rounding: .toNearest)))).aprText == "APR 2.15%")
        #expect(ValidatorViewModel(row: .mock(apr: .apr(value: nil))).aprText == "APR ")
    }
}
