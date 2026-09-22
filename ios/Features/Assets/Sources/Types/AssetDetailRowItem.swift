// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemListRow
import Primitives

public enum AssetDetailRowAction {
    case price
    case network(AssetNetworkDestination)
    case stake
    case earn
    case explorer(URL)
    case priceAlerts
    case pin
    case enable
}

public enum AssetNetworkDestination {
    case asset(Asset)
    case assets(Chain)
}

public enum AssetDetailRowContent {
    case item(ListItemModel)
    case row(GemListRow)
}

public struct AssetDetailRowItem: Identifiable {
    public let id: String
    public let content: AssetDetailRowContent
    public let action: AssetDetailRowAction?
    public let accessibilityIdentifier: String?

    public init(id: String, content: AssetDetailRowContent, action: AssetDetailRowAction? = nil, accessibilityIdentifier: String? = nil) {
        self.id = id
        self.content = content
        self.action = action
        self.accessibilityIdentifier = accessibilityIdentifier
    }
}

public struct AssetDetailSectionItem: Identifiable {
    public let id: String
    public let title: String?
    public let rows: [AssetDetailRowItem]

    public init(id: String, title: String?, rows: [AssetDetailRowItem]) {
        self.id = id
        self.title = title
        self.rows = rows
    }
}
