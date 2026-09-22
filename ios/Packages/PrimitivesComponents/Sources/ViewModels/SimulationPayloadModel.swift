// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemSimulationPayloadRow
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
        onSelectAddress: (@MainActor @Sendable (String) -> Void)? = nil,
    ) -> [SimulationPayloadFieldViewModel] {
        rows.map { fieldModel(for: $0, onSelectAddress: onSelectAddress) }
    }

    private func fieldModel(
        for row: GemSimulationPayloadRow,
        onSelectAddress: (@MainActor @Sendable (String) -> Void)?,
    ) -> SimulationPayloadFieldViewModel {
        guard case let .address(_, copy, explorer) = row.value else {
            return SimulationPayloadFieldViewModel(row: row)
        }
        return SimulationPayloadFieldViewModel(
            row: row,
            kind: .address(ExplorerContextData(copyValue: copy.copyValue, explorerLink: explorer.toPrimitives())),
            onSelect: selectAddress(copy.value, onSelectAddress: onSelectAddress),
        )
    }

    private func selectAddress(
        _ address: String,
        onSelectAddress: (@MainActor @Sendable (String) -> Void)?,
    ) -> (@MainActor @Sendable () -> Void)? {
        guard let onSelectAddress else { return nil }
        return { onSelectAddress(address) }
    }
}
