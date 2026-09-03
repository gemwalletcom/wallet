// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension FeePriority: Identifiable {
    public var id: String {
        rawValue
    }
}

public extension FeePriority {
    init(id: String) throws {
        if let priority = FeePriority(rawValue: id) {
            self = priority
        } else {
            throw AnyError("invalid priority: \(id)")
        }
    }
}
