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
        let notification = InAppNotification.mock(createdAt: Date(timeIntervalSince1970: 1_700_000_000))

        #expect(try Primitives.InAppNotification(notification.json()).createdAt == notification.createdAt)
    }

    @Test
    func roundTripsNestedRecord() throws {
        let notification = InAppNotification.mock(item: .mock(icon: .emoji(.gift)))
        let decoded = try Primitives.InAppNotification(notification.json())
        #expect(decoded.item.icon == notification.item.icon)
        #expect(decoded.item.id == notification.item.id)
    }
}
