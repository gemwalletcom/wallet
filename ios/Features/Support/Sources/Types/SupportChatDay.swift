// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

struct SupportChatDay: Identifiable {
    let date: Date
    let title: String
    let groups: [SupportChatGroup]

    var id: Date { date }
}

struct SupportChatGroup: Identifiable {
    let sender: SupportMessageSender
    let messages: [SupportMessageBubbleViewModel]

    var id: String {
        messages.first?.id ?? ""
    }
}
