// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct SupportChatStoreTests {
    @Test
    func failPendingLeavesTheMessagesStillInFlightAlone() throws {
        let db = DB.mock()
        let store = SupportChatStore(db: db)

        try store.addMessages([
            .mock(id: "abandoned", status: .sending),
            .mock(id: "in-flight", status: .sending),
            .mock(id: "delivered", status: .sent),
        ])

        try store.failPending(exceptIds: ["in-flight"])

        #expect(try statuses(db) == ["abandoned": .failed, "in-flight": .sending, "delivered": .sent])
    }

    @Test
    func failPendingWithNothingInFlightFailsEverySendingMessage() throws {
        let db = DB.mock()
        let store = SupportChatStore(db: db)

        try store.addMessages([.mock(id: "abandoned", status: .sending)])

        try store.failPending(exceptIds: [])

        #expect(try statuses(db) == ["abandoned": .failed])
    }

    private func statuses(_ db: DB) throws -> [String: SupportMessageStatus] {
        try db.dbQueue
            .read { try SupportMessagesQuery().fetch($0) }
            .reduce(into: [:]) { $0[$1.id] = $1.status }
    }
}
