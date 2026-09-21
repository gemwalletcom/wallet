// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
@testable import Support
@testable import SupportTestKit
import Testing

struct SupportChatDayBuilderTests {
    @Test
    func groupsMessagesByDaySortedAscending() throws {
        let calendar = Calendar.current
        let days = try SupportChatDayBuilder.mock(messages: [
            .mock(id: "a", createdAt: #require(calendar.date(from: DateComponents(year: 2026, month: 1, day: 2, hour: 12)))),
            .mock(id: "b", createdAt: #require(calendar.date(from: DateComponents(year: 2026, month: 1, day: 1, hour: 12)))),
            .mock(id: "c", createdAt: #require(calendar.date(from: DateComponents(year: 2026, month: 1, day: 3, hour: 12)))),
        ]).build()

        #expect(days.count == 3)
        #expect(days[0].groups[0].messages.map(\.id) == ["b"])
        #expect(days[1].groups[0].messages.map(\.id) == ["a"])
        #expect(days[2].groups[0].messages.map(\.id) == ["c"])
    }

    @Test
    func keepsSameDayDifferentHoursTogether() throws {
        let calendar = Calendar.current
        let days = try SupportChatDayBuilder.mock(messages: [
            .mock(id: "a", createdAt: #require(calendar.date(from: DateComponents(year: 2026, month: 1, day: 1, hour: 8)))),
            .mock(id: "b", createdAt: #require(calendar.date(from: DateComponents(year: 2026, month: 1, day: 1, hour: 20)))),
        ]).build()

        #expect(days.count == 1)
        #expect(days[0].groups.count == 1)
        #expect(days[0].groups[0].messages.map(\.id) == ["a", "b"])
    }

    @Test
    func groupsCarryTheSenderAndTheBubbles() throws {
        let date = try #require(Calendar.current.date(from: DateComponents(year: 2026, month: 1, day: 1, hour: 12)))
        let groups = SupportChatDayBuilder.mock(messages: [
            .mock(id: "a", sender: .user, createdAt: date),
            .mock(id: "b", sender: .agent(.mock(name: "Gemma")), createdAt: date),
            .mock(id: "c", sender: .agent(.mock(name: "Gemma")), createdAt: date),
        ]).build()[0].groups

        #expect(groups.map(\.sender) == [.user, .agent(.mock(name: "Gemma"))])
        #expect(groups.map { $0.messages.map(\.id) } == [["a"], ["b", "c"]])
    }

    @Test
    func emptyMessagesProduceNoDays() {
        #expect(SupportChatDayBuilder.mock().build().isEmpty)
    }
}
