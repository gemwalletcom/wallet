// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

struct JsonCodableTests {
    @Test
    func roundTripsTaggedEnum() throws {
        let stakeType = Primitives.StakeType.freeze(.bandwidth)

        #expect(try Primitives.StakeType(stakeType.json()) == stakeType)
    }

    @Test
    func roundTripsDate() throws {
        let message = Primitives.SupportMessage.mock(createdAt: Date(timeIntervalSince1970: 1_700_000_000))

        #expect(try Primitives.SupportMessage(message.json()).createdAt == message.createdAt)
    }

    @Test
    func roundTripsNestedRecord() throws {
        let delegation = Primitives.Delegation.mock()
        let decoded = try Primitives.Delegation(delegation.json())
        #expect(decoded.base.delegationId == delegation.base.delegationId)
        #expect(decoded.validator.id == delegation.validator.id)
    }
}
