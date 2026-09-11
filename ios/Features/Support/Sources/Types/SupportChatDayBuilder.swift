// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import func Gemstone.supportChatGroups

struct SupportChatDayBuilder {
    let messages: [SupportMessage]
    let retryAction: (SupportMessage) -> Void
    let imageAction: (SupportMessageImage) -> Void

    func build() -> [SupportChatDay] {
        Dictionary(grouping: messages) { Calendar.current.startOfDay(for: $0.createdAt) }
            .sorted { $0.key < $1.key }
            .map { day in
                SupportChatDay(date: day.key, groups: groups(from: day.value))
            }
    }
}

// MARK: - Private

private extension SupportChatDayBuilder {
    func groups(from messages: [SupportMessage]) -> [SupportChatGroup] {
        supportChatGroups(messages: messages.map { $0.map() }).map { group in
            SupportChatGroup(
                sender: group.sender.map(),
                messages: group.messages.map { SupportMessageBubbleViewModel(message: $0.map(), retryAction: retryAction, imageAction: imageAction) },
            )
        }
    }
}
