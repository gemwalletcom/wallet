// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

struct JsonCodableTests {
    @Test
    func roundTripsTaggedEnum() throws {
        let stakeData = Primitives.TronStakeData.unfreeze([TronUnfreeze(resource: .bandwidth, amount: 1)])

        #expect(try Primitives.TronStakeData(stakeData.json()) == stakeData)
    }

    @Test
    func roundTripsDate() throws {
        let message = Primitives.SupportMessage.mock(createdAt: Date(timeIntervalSince1970: 1_700_000_000))

        #expect(try Primitives.SupportMessage(message.json()).createdAt == message.createdAt)
    }

    @Test
    func roundTripsNestedRecord() throws {
        let message = Primitives.SupportMessage.mock(sender: .agent(.mock(name: "Gemma")))
        let decoded = try Primitives.SupportMessage(message.json())
        #expect(decoded.sender == message.sender)
        #expect(decoded.id == message.id)
    }
}
