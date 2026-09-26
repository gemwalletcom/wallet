// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemDayBoundaries
import func Gemstone.supportChatGroups
import GemstonePrimitives
import Primitives
import PrimitivesComponents

struct SupportChatDayBuilder {
    let messages: [SupportMessage]

    func build() -> [SupportChatDay] {
        let boundaries = GemDayBoundaries.current
        return boundaries.sections(days: messages.map(\.createdAt.gemDay), newestFirst: false).map { section in
            let date = section.day.date
            return SupportChatDay(
                date: date,
                title: TransactionDateFormatter(date: date, boundaries: boundaries).section,
                groups: supportChatGroups(messages: section.positions.map { messages[Int($0)].toGem() }),
            )
        }
    }
}
