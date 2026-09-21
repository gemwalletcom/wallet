// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct AssetConfiguration: Sendable {
    public let isEnabled: Bool?
    public let isPinned: Bool?

    public init(isEnabled: Bool?, isPinned: Bool?) {
        self.isEnabled = isEnabled
        self.isPinned = isPinned
    }
}
