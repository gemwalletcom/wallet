// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemSimulationPayloadRow
import Primitives

public struct SimulationPayloadFieldViewModel: Identifiable {
    public let row: GemSimulationPayloadRow
    public let relativeDateFormatter: RelativeDateFormatter
    public let explorerItem: ContextMenuItemType?

    public init(
        row: GemSimulationPayloadRow,
        relativeDateFormatter: RelativeDateFormatter = RelativeDateFormatter(),
        explorerItem: ContextMenuItemType? = nil,
    ) {
        self.row = row
        self.relativeDateFormatter = relativeDateFormatter
        self.explorerItem = explorerItem
    }

    public var id: GemSimulationPayloadRow {
        row
    }

    public var listItem: ListItemModel {
        ListItemModel(title: title, subtitle: subtitle)
    }

    public var title: String {
        row.title.text
    }

    public var subtitle: String {
        switch row.value {
        case let .text(text): text
        case let .address(display, _): display
        case let .timestamp(unixMs): relativeDateFormatter.string(from: Date(timeIntervalSince1970: TimeInterval(unixMs) / 1000))
        }
    }

    public var contextMenuItems: [ContextMenuItemType] {
        switch row.value {
        case .text, .timestamp: []
        case let .address(_, address): [.copy(value: address)] + (explorerItem.map { [$0] } ?? [])
        }
    }
}
