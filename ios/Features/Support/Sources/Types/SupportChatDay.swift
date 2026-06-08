// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

struct SupportChatDay: Identifiable {
    let id: String
    let date: Date
    let groups: [SupportChatGroup]
}

struct SupportChatGroup: Identifiable {
    enum Kind {
        case user(messages: [SupportMessageBubbleViewModel])
        case agent(header: SupportAgentHeader, messages: [SupportMessageBubbleViewModel])
    }

    let kind: Kind
    let isLast: Bool

    var id: String {
        switch kind {
        case let .user(messages): "user-\(messages.first?.id ?? "")"
        case let .agent(_, messages): "agent-\(messages.first?.id ?? "")"
        }
    }
}

struct SupportAgentHeader {
    let name: String

    init(agent: SupportAgent) {
        name = agent.name
    }
}
