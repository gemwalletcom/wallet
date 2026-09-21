// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Stake
import Testing

struct ValidatorViewModelTests {
    @Test func aprText() {
        #expect(ValidatorViewModel(row: .mock(apr: .mock(value: 2.15, unit: .percent, notation: .plain))).aprText == "APR 2.15%")
        #expect(ValidatorViewModel(row: .mock()).aprText == "APR ")
    }
}
