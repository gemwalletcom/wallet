// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum WalletSearchTag: Hashable, Codable, Sendable {
    case all
    case filter(AssetTag)
    case list(String)
    case chain(Chain)
}

public extension WalletSearchTag {
    var includesPerpetuals: Bool {
        switch self {
        case .filter, .chain: false
        case .all, .list: true
        }
    }

    var isList: Bool {
        switch self {
        case .list: true
        case .all, .filter, .chain: false
        }
    }

    var isAll: Bool {
        switch self {
        case .all: true
        case .filter, .list, .chain: false
        }
    }

    var chain: Chain? {
        switch self {
        case let .chain(chain): chain
        case .all, .filter, .list: nil
        }
    }

    func searchKey(query: String) -> String {
        apiTag.map { query.isEmpty ? "tag:\($0)" : query } ?? query
    }

    var apiTag: String? {
        switch self {
        case .all, .chain: nil
        case let .filter(tag): tag.rawValue
        case let .list(value): value
        }
    }
}
