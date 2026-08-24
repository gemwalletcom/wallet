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

    func searchKey(query: String) -> String {
        apiTag.map { query.isEmpty ? "tag:\($0)" : query } ?? query
    }

    var apiTag: String? {
        switch self {
        case .all: nil
        case let .list(value): value
        }
    }
}
