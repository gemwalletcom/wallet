// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemSimulationPayloadRow
import Localization
import Primitives

public struct SimulationPayloadModel: Sendable {
    public let primaryFields: [GemSimulationPayloadRow]
    public let secondaryFields: [GemSimulationPayloadRow]

    public init(
        primaryFields: [GemSimulationPayloadRow],
        secondaryFields: [GemSimulationPayloadRow],
    ) {
        self.primaryFields = primaryFields
        self.secondaryFields = secondaryFields
    }

    public var hasFields: Bool { primaryFields.isNotEmpty || secondaryFields.isNotEmpty }
    public var hasDetails: Bool { secondaryFields.isNotEmpty }

    public func fieldModels(
        for rows: [GemSimulationPayloadRow],
        explorerLink: (String) -> BlockExplorerLink,
        onOpenURL: @escaping (URL) -> Void,
    ) -> [SimulationPayloadFieldViewModel] {
        rows.map { row in
            let explorerItem: ContextMenuItemType? = switch row.value {
            case let .address(_, address): explorerMenuItem(link: explorerLink(address), onOpenURL: onOpenURL)
            case .text, .timestamp: nil
            }
            return SimulationPayloadFieldViewModel(row: row, explorerItem: explorerItem)
        }
    }

    private func explorerMenuItem(
        link: BlockExplorerLink,
        onOpenURL: @escaping (URL) -> Void,
    ) -> ContextMenuItemType {
        .url(title: Localized.Transaction.viewOn(link.name), onOpen: {
            if let url = URL(string: link.link) {
                onOpenURL(url)
            }
        })
    }
}
