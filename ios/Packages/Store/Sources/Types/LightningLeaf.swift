// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct LightningLeaf: Equatable, Sendable {
    public let id: String
    public let data: String

    public init(id: String, data: String) {
        self.id = id
        self.data = data
    }
}
