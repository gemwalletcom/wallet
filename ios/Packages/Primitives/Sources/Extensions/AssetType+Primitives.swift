// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension AssetType {
    init(id: String) throws {
        if let type = AssetType(rawValue: id) {
            self = type
        } else {
            throw AnyError("invalid asset type: \(id)")
        }
    }
}
