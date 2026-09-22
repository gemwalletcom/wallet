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
        onSelectAddress: (@MainActor @Sendable (String) -> Void)? = nil,
    ) -> [SimulationPayloadFieldViewModel] {
        rows.map { fieldModel(for: $0, explorerLink: explorerLink, onOpenURL: onOpenURL, onSelectAddress: onSelectAddress) }
    }

    private func fieldModel(
        for row: GemSimulationPayloadRow,
        explorerLink: (String) -> BlockExplorerLink,
        onOpenURL: @escaping (URL) -> Void,
        onSelectAddress: (@MainActor @Sendable (String) -> Void)?,
    ) -> SimulationPayloadFieldViewModel {
        guard case let .address(_, address) = row.value else {
            return SimulationPayloadFieldViewModel(row: row)
        }
        return SimulationPayloadFieldViewModel(
            row: row,
            explorerItem: explorerMenuItem(link: explorerLink(address), onOpenURL: onOpenURL),
            onSelect: selectAddress(address, onSelectAddress: onSelectAddress),
        )
    }

    private func selectAddress(
        _ address: String,
        onSelectAddress: (@MainActor @Sendable (String) -> Void)?,
    ) -> (@MainActor @Sendable () -> Void)? {
        guard let onSelectAddress else { return nil }
        return { onSelectAddress(address) }
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
