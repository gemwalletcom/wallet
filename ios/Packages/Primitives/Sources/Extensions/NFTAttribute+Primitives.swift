// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension NFTAttribute: Identifiable {
    public var id: String {
        name + value
    }
}

public extension NFTAttribute {
    init(name: String, value: String, percentage: Double? = nil) {
        self.init(name: name, value: value, valueType: .string, percentage: percentage)
    }
}
