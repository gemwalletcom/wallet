// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives
import Testing

struct BigInt_PrimitivesTests {
    @Test
    func testIsBetween() {
        #expect(BigInt(100).isBetween(99, and: 110))
        #expect(BigInt(100).isBetween(110, and: 99) == false)
        #expect(BigInt(100).isBetween(0, and: 10) == false)
        #expect(BigInt(0).isBetween(0, and: 1))
    }

    @Test
    func fromString() {
        #expect(BigInt.from(string: "111").description == "111")
        #expect(BigInt.from(string: "0.1").description == "0")
    }
}
