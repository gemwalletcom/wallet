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

public extension SimulationResult {
    static func mockPermitBatch(
        warnings: [SimulationWarning] = [],
        header: SimulationHeader? = nil,
    ) -> SimulationResult {
        .mock(
            warnings: warnings,
            payload: [
                .standard(kind: .spender, value: "0x3333333333333333333333333333333333333333", fieldType: .address, display: .primary),
                .standard(kind: .value, value: "Unlimited", fieldType: .text, display: .primary),
                .standard(kind: .contract, value: "0x000000000022D473030F116dDEE9F6B43aC78BA3", fieldType: .address, display: .secondary),
                .standard(kind: .method, value: "Permit Batch", fieldType: .text, display: .secondary),
            ],
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
