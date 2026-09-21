// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemSimulationPayloadRow
import Primitives

public struct SimulationPayloadFieldViewModel: Identifiable {
    public let row: GemSimulationPayloadRow
    public let explorerItem: ContextMenuItemType?

    public init(
        row: GemSimulationPayloadRow,
        explorerItem: ContextMenuItemType? = nil,
    ) {
        self.row = row
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
        case let .timestamp(unixMs): TransactionDateFormatter(unixMilliseconds: unixMs).row
        }
    }

    public var contextMenuItems: [ContextMenuItemType] {
        switch row.value {
        case .text, .timestamp: []
        case let .address(_, address): [.copy(value: address)] + (explorerItem.map { [$0] } ?? [])
        }
    }
}
