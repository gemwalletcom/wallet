// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Testing

struct PerpetualIdTests {
    @Test
    func fromIdAndBackToId() throws {
        let perpetual = try PerpetualId.from(id: "hypercore_BTC")
        #expect(perpetual == PerpetualId(provider: .hypercore, symbol: "BTC"))
        #expect(perpetual.identifier == "hypercore_BTC")

        let symbolWithUnderscore = try PerpetualId.from(id: "hypercore_BTC_PERP")
        #expect(symbolWithUnderscore.provider == .hypercore)
        #expect(symbolWithUnderscore.symbol == "BTC_PERP")
        #expect(symbolWithUnderscore.identifier == "hypercore_BTC_PERP")

        #expect(throws: Error.self) { try PerpetualId.from(id: "hypercore") }
        #expect(throws: Error.self) { try PerpetualId.from(id: "unknownprovider_BTC") }
        #expect(throws: Error.self) { try PerpetualId.from(id: "") }

        let encoded = try JSONEncoder().encode(perpetual)
        #expect(String(data: encoded, encoding: .utf8) == "\"hypercore_BTC\"")
        #expect(try JSONDecoder().decode(PerpetualId.self, from: encoded) == perpetual)
    }
}
