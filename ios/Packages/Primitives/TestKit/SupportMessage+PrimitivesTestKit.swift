// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension SupportMessage {
    static func mock(
        id: String = "1",
        content: String = "content",
        sender: SupportMessageSender = .user,
        status: SupportMessageStatus = .sent,
        createdAt: Date = Date(timeIntervalSince1970: 0),
        images: [SupportMessageImage] = [],
    ) -> SupportMessage {
        SupportMessage(
            id: id,
            content: content,
            sender: sender,
            status: status,
            createdAt: createdAt,
            images: images,
        )
    }
}

public extension SupportAgent {
    static func mock(name: String = "Gemma") -> SupportAgent {
        SupportAgent(name: name)
    }
}
