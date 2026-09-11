// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum WalletSearchTag: Hashable, Codable, Sendable {
    case all
    case list(String)
}

public extension WalletSearchTag {
    var isList: Bool {
        switch self {
        case .list: true
        case .all: false
        }
    }

    var isAll: Bool {
        switch self {
        case .all: true
        case .list: false
        }
    }
}
