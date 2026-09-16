// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemDayBoundaries
import GemstonePrimitives
import PrimitivesComponents
import Primitives
import func Gemstone.supportChatGroups

struct SupportChatDayBuilder {
    let messages: [SupportMessage]
    let retryAction: (SupportMessage) -> Void
    let imageAction: (SupportMessageImage) -> Void

    func build() -> [SupportChatDay] {
        let boundaries = GemDayBoundaries.current
        return Dictionary(grouping: messages) { Calendar.current.startOfDay(for: $0.createdAt) }
            .sorted { $0.key < $1.key }
            .map { day in
                SupportChatDay(
                    date: day.key,
                    title: TransactionDateFormatter(date: day.key, boundaries: boundaries).section,
                    groups: groups(from: day.value),
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
                messages: group.messages.map { SupportMessageBubbleViewModel(message: $0.toPrimitives(), retryAction: retryAction, imageAction: imageAction) },
            )
        }
    }
}
