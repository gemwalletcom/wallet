// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import GemstonePrimitives
import Primitives
import Testing

// Guards the JSON bridge: every type that crosses the FFI as JSON must survive a
// round-trip through the coders in `JsonCodable`, including the representations
// that differ between serde and Codable (dates, big integers, tagged enums).
struct JsonCodableTests {
    @Test
    func roundTripsTaggedEnum() throws {
        let stakeType = Primitives.StakeType.freeze(.bandwidth)

        #expect(try Primitives.StakeType(stakeType.json()) == stakeType)
    }

    @Test
    func roundTripsDate() throws {
        // Rust emits RFC3339; a bare JSONDecoder would read it as a timestamp and fail.
        let value = Primitives.ChartDateValue(date: Date(timeIntervalSince1970: 1_700_000_000), value: 42)

        #expect(try Primitives.ChartDateValue(value.json()).date == value.date)
    }

    @Test
    func roundTripsNestedRecord() throws {
        let field = Primitives.SimulationPayloadField(
            kind: .value,
            label: "Amount",
            value: "1.0",
            fieldType: .text,
            display: .primary,
        )

        #expect(try Primitives.SimulationPayloadField(field.json()) == field)
    }
}
