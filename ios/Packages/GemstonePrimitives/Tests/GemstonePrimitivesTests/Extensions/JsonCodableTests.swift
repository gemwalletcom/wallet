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
        let asset = Primitives.AssetBasic.mock()
        let decoded = try Primitives.AssetBasic(asset.json())
        #expect(decoded.asset.id == asset.asset.id)
        #expect(decoded.score.rank == asset.score.rank)
    }
}
