// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemDayBoundaries
import func Gemstone.supportChatGroups
import GemstonePrimitives
import Primitives
import PrimitivesComponents

struct SupportChatDayBuilder {
    let messages: [SupportMessage]
    let retryAction: (SupportMessage) -> Void
    let imageAction: (SupportMessageImage) -> Void

    func build() -> [SupportChatDay] {
        let boundaries = GemDayBoundaries.current
        return boundaries.sections(days: messages.map(\.createdAt.gemDay), newestFirst: false).map { section in
            let date = section.day.date
            return SupportChatDay(
                date: date,
                title: TransactionDateFormatter(date: date, boundaries: boundaries).section,
                groups: groups(from: section.positions.map { messages[Int($0)] }),
            )
        }
    }
}

// MARK: - Private

private extension SupportChatDayBuilder {
    func groups(from messages: [SupportMessage]) -> [SupportChatGroup] {
        supportChatGroups(messages: messages.map { $0.toGem() }).map { group in
            SupportChatGroup(
                sender: group.sender.toPrimitives(),
                messages: group.rows.map { SupportMessageBubbleViewModel(row: $0, retryAction: retryAction, imageAction: imageAction) },
            )
        }
    }
}
