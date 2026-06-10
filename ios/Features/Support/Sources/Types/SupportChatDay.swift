// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

struct SupportChatDay: Identifiable {
    let date: Date
    let groups: [SupportChatGroup]

    var id: Date { date }
}

struct SupportChatGroup: Identifiable {
    enum Kind {
        case user(messages: [SupportMessageBubbleViewModel])
        case agent(name: String, messages: [SupportMessageBubbleViewModel])
    }

    let kind: Kind

    var id: String {
        switch kind {
        case let .user(messages), let .agent(_, messages):
            messages.first?.id ?? ""
        }
    }
}
