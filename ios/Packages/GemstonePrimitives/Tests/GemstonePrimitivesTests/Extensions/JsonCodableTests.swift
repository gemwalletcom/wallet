// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

struct JsonCodableTests {
    @Test
    func roundTripsTaggedEnum() throws {
        let event = Primitives.StreamEvent.nft(StreamWalletUpdate(walletId: .mock()))

        guard case let .nft(update) = try Primitives.StreamEvent(event.json()) else {
            Issue.record("the tag did not survive the round trip")
            return
        }
        #expect(update.walletId == WalletId.mock())
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
