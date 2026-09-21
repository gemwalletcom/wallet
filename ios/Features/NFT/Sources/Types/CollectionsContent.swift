// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization

public struct CollectionsContent: Sendable {
    public let items: [GridPosterViewItem]
    public let unverifiedCount: String?

    public var unverifiedListItem: ListItemModel? {
        unverifiedCount.map { ListItemModel(title: Localized.Asset.Verification.unverified, subtitle: $0) }
    }

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
