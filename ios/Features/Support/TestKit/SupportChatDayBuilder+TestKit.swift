// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
@testable import Support

extension SupportChatDayBuilder {
    static func mock(messages: [SupportMessage] = []) -> SupportChatDayBuilder {
        SupportChatDayBuilder(messages: messages)
    }
}
