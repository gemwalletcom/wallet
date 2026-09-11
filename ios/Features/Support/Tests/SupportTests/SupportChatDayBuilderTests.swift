// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
@testable import Support
import Testing

struct SupportChatDayBuilderTests {
    @Test
    func groupsMessagesByDaySortedAscending() {
        let days = build([
            message("a", .user, day: 2),
            message("b", .user, day: 1),
            message("c", .user, day: 3),
        ])

        #expect(days.count == 3)
        #expect(days[0].groups[0].bubbleIds == ["b"])
        #expect(days[1].groups[0].bubbleIds == ["a"])
        #expect(days[2].groups[0].bubbleIds == ["c"])
    }

    @Test
    func keepsSameDayDifferentHoursTogether() {
        let days = build([
            message("a", .user, day: 1, hour: 8),
            message("b", .user, day: 1, hour: 20),
        ])

        #expect(days.count == 1)
        #expect(days[0].groups.count == 1)
        #expect(days[0].groups[0].bubbleIds == ["a", "b"])
    }

    @Test
    func groupsCarryTheSenderAndTheBubbles() {
        let groups = build([
            message("a", .user, day: 1),
            message("b", agent("Gemma"), day: 1),
            message("c", agent("Gemma"), day: 1),
        ])[0].groups

        #expect(groups.map(\.sender) == [.user, agent("Gemma")])
        #expect(groups.map(\.bubbleIds) == [["a"], ["b", "c"]])
    }

    @Test
    func emptyMessagesProduceNoDays() {
        #expect(build([]).isEmpty)
    }
}

// MARK: - Helpers

private extension SupportChatDayBuilderTests {
    func build(_ messages: [SupportMessage]) -> [SupportChatDay] {
        SupportChatDayBuilder(messages: messages, retryAction: { _ in }, imageAction: { _ in }).build()
    }

    func message(_ id: String, _ sender: SupportMessageSender, day: Int, hour: Int = 12) -> SupportMessage {
        .mock(id: id, sender: sender, createdAt: date(day: day, hour: hour))
    }

    func agent(_ name: String) -> SupportMessageSender {
        .agent(.mock(name: name))
    }

    func date(day: Int, hour: Int) -> Date {
        Calendar.current.date(from: DateComponents(year: 2026, month: 1, day: day, hour: hour))!
    }
}

private extension SupportChatGroup {
    var bubbleIds: [String] {
        messages.map(\.id)
    }
}
