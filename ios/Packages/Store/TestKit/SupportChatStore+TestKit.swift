// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store

public extension SupportChatStore {
    static func mock(db: DB = .mock()) -> Self {
        SupportChatStore(db: db)
    }
}
