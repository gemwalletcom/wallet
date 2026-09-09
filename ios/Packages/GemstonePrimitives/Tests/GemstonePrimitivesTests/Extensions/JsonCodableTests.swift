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
        let createdAt = Date(timeIntervalSince1970: 1_700_000_000)
        let event = Primitives.StreamEvent.inAppNotification(StreamNotificationUpdate(walletId: .mock(), notification: .mock(createdAt: createdAt)))

        guard case let .inAppNotification(update) = try Primitives.StreamEvent(event.json()) else {
            Issue.record("the tag did not survive the round trip")
            return
        }
        #expect(update.notification.createdAt == createdAt)
    }

    @Test
    func roundTripsNestedRecord() throws {
        let item = CoreListItem.mock(id: "reward", icon: .emoji(.gift))
        let event = Primitives.StreamEvent.inAppNotification(StreamNotificationUpdate(walletId: .mock(), notification: .mock(item: item)))

        guard case let .inAppNotification(update) = try Primitives.StreamEvent(event.json()) else {
            Issue.record("the tag did not survive the round trip")
            return
        }
        #expect(update.notification.item.icon == item.icon)
        #expect(update.notification.item.id == item.id)
    }
}
