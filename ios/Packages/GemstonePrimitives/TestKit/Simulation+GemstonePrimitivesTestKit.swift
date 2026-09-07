// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.SimulationBalanceChange
import struct Gemstone.SimulationHeader
import struct Gemstone.SimulationPayloadField
import enum Gemstone.SimulationPayloadFieldDisplay
import enum Gemstone.SimulationPayloadFieldKind
import enum Gemstone.SimulationPayloadFieldType
import struct Gemstone.SimulationResult
import struct Gemstone.SimulationWarning
import Foundation

public extension SimulationResult {
    static func mock(
        warnings: [SimulationWarning] = [],
        balanceChanges: [SimulationBalanceChange] = [],
        payload: [SimulationPayloadField] = [],
        header: SimulationHeader? = nil,
    ) -> SimulationResult {
        SimulationResult(
            warnings: warnings,
            balanceChanges: balanceChanges,
            payload: payload,
            header: header,
        )
    }
}

public extension SimulationPayloadField {
    static func standard(
        kind: SimulationPayloadFieldKind,
        value: String,
        fieldType: SimulationPayloadFieldType,
        display: SimulationPayloadFieldDisplay = .secondary,
    ) -> Self {
        if kind == .custom {
            preconditionFailure("Use custom(label:value:fieldType:) for custom payload fields")
        }
        return Self(kind: kind, label: nil, value: value, fieldType: fieldType, display: display)
    }

    static func custom(
        label: String,
        value: String,
        fieldType: SimulationPayloadFieldType,
        display: SimulationPayloadFieldDisplay = .secondary,
    ) -> Self {
        precondition(!label.isEmpty, "Custom payload fields require a label")
        return Self(kind: .custom, label: label, value: value, fieldType: fieldType, display: display)
    }
}
