// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemSimulationPayloadRow
import Primitives

public enum SimulationPayloadFieldKind: Sendable, Equatable {
    case address(ExplorerContextData)
    case plain
}

public struct SimulationPayloadFieldViewModel: Identifiable {
    public let row: GemSimulationPayloadRow
    public let kind: SimulationPayloadFieldKind
    public let onSelect: (@MainActor @Sendable () -> Void)?

    public init(
        row: GemSimulationPayloadRow,
        kind: SimulationPayloadFieldKind = .plain,
        onSelect: (@MainActor @Sendable () -> Void)? = nil,
    ) {
        self.row = row
        self.kind = kind
        self.onSelect = onSelect
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
        case let .address(display, _, _): display
        case let .timestamp(unixMs): TransactionDateFormatter(unixMilliseconds: unixMs).row
        }
    }
}

public extension SimulationPayloadFieldViewModel {
    static func models(
        for rows: [GemSimulationPayloadRow],
        onSelectAddress: (@MainActor @Sendable (String) -> Void)? = nil,
    ) -> [SimulationPayloadFieldViewModel] {
        rows.map { row -> SimulationPayloadFieldViewModel in
            guard case let .address(_, copy, explorer) = row.value else {
                return SimulationPayloadFieldViewModel(row: row)
            }
            let address = copy.value
            var onSelect: (@MainActor @Sendable () -> Void)?
            if let onSelectAddress {
                onSelect = { onSelectAddress(address) }
            }
            return SimulationPayloadFieldViewModel(
                row: row,
                kind: .address(ExplorerContextData(copyValue: copy.copyValue, explorerLink: explorer.toPrimitives())),
                onSelect: onSelect,
            )
        }
    }
}
