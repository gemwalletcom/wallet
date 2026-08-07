// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct CollectionsContent: Sendable {
    public let items: [GridPosterViewItem]
    public let unverifiedCount: String?

    public var isEmpty: Bool {
        items.isEmpty && unverifiedCount == nil
    }

    public init(
        items: [GridPosterViewItem],
        unverifiedCount: String? = nil,
    ) {
        self.items = items
        self.unverifiedCount = unverifiedCount
    }
}
