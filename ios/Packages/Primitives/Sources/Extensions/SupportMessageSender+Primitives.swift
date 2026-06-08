// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension SupportMessageSender {
    var isAgent: Bool {
        switch self {
        case .agent: true
        case .user: false
        }
    }
}
