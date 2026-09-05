// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import GemstonePrimitives
import Primitives
import Testing

struct JsonCodableTests {
    @Test
    func roundTripsTaggedEnum() throws {
        let stakeType = Primitives.StakeType.freeze(.bandwidth)

        #expect(try Primitives.StakeType(stakeType.json()) == stakeType)
    }

    @Test
    func roundTripsDate() throws {
        let value = Primitives.ChartDateValue(date: Date(timeIntervalSince1970: 1_700_000_000), value: 42)

        #expect(try Primitives.ChartDateValue(value.json()).date == value.date)
    }

    @Test
    func roundTripsNestedRecord() throws {
        let address = Primitives.ChainAddress(chain: .ethereum, address: "0x1")
        let decoded = try Primitives.ChainAddress(address.json())
        #expect(decoded.chain == address.chain)
        #expect(decoded.address == address.address)
    }
}
