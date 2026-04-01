// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.SimulationBalanceChange
import struct Gemstone.SimulationHeader
import struct Gemstone.SimulationPayloadField
import enum Gemstone.SimulationPayloadFieldDisplay
import enum Gemstone.SimulationPayloadFieldKind
import enum Gemstone.SimulationPayloadFieldType
import struct Gemstone.SimulationResult
import enum Gemstone.SimulationSeverity
import struct Gemstone.SimulationWarning
import struct Gemstone.SimulationWarningApproval
import enum Gemstone.SimulationWarningType
import Primitives

public extension Gemstone.SimulationSeverity {
    func map() -> Primitives.SimulationSeverity {
        switch self {
        case .low: .low
        case .warning: .warning
        case .critical: .critical
        }
    }
}

public extension Gemstone.SimulationWarningType {
    func map() throws -> Primitives.SimulationWarningType {
        switch self {
        case let .tokenApproval(approval): try .tokenApproval(approval.map())
        case .suspiciousSpender: .suspiciousSpender
        case .externallyOwnedSpender: .externallyOwnedSpender
        case let .nftCollectionApproval(assetId): try .nftCollectionApproval(AssetId(id: assetId))
        case let .permitApproval(approval): try .permitApproval(approval.map())
        case let .permitBatchApproval(value): .permitBatchApproval(value?.description)
        case .validationError: .validationError
        }
    }
}

public extension Gemstone.SimulationWarningApproval {
    func map() throws -> Primitives.SimulationWarningApproval {
        try Primitives.SimulationWarningApproval(assetId: AssetId(id: assetId), value: value?.description)
    }
}

public extension Gemstone.SimulationPayloadFieldType {
    func map() -> Primitives.SimulationPayloadFieldType {
        switch self {
        case .text: .text
        case .address: .address
        case .timestamp: .timestamp
        }
    }
}

public extension Primitives.SimulationPayloadFieldType {
    func map() -> Gemstone.SimulationPayloadFieldType {
        switch self {
        case .text: .text
        case .address: .address
        case .timestamp: .timestamp
        }
    }
}

public extension Gemstone.SimulationPayloadFieldDisplay {
    func map() -> Primitives.SimulationPayloadFieldDisplay {
        switch self {
        case .primary: .primary
        case .secondary: .secondary
        }
    }
}

public extension Primitives.SimulationPayloadFieldDisplay {
    func map() -> Gemstone.SimulationPayloadFieldDisplay {
        switch self {
        case .primary: .primary
        case .secondary: .secondary
        }
    }
}

public extension Gemstone.SimulationPayloadFieldKind {
    func map() -> Primitives.SimulationPayloadFieldKind {
        switch self {
        case .contract: .contract
        case .method: .method
        case .token: .token
        case .spender: .spender
        case .value: .value
        case .custom: .custom
        }
    }
}

public extension Primitives.SimulationPayloadFieldKind {
    func map() -> Gemstone.SimulationPayloadFieldKind {
        switch self {
        case .contract: .contract
        case .method: .method
        case .token: .token
        case .spender: .spender
        case .value: .value
        case .custom: .custom
        }
    }
}

public extension Gemstone.SimulationWarning {
    func map() throws -> Primitives.SimulationWarning {
        try Primitives.SimulationWarning(severity: severity.map(), warning: warning.map(), message: message)
    }
}

public extension Gemstone.SimulationBalanceChange {
    func map() throws -> Primitives.SimulationBalanceChange {
        try Primitives.SimulationBalanceChange(assetId: AssetId(id: assetId), value: value)
    }
}

public extension Gemstone.SimulationPayloadField {
    func map() -> Primitives.SimulationPayloadField {
        Primitives.SimulationPayloadField(kind: kind.map(), label: label, value: value, fieldType: fieldType.map(), display: display.map())
    }
}

public extension Primitives.SimulationPayloadField {
    func map() -> Gemstone.SimulationPayloadField {
        Gemstone.SimulationPayloadField(kind: kind.map(), label: label, value: value, fieldType: fieldType.map(), display: display.map())
    }
}

public extension Gemstone.SimulationHeader {
    func map() throws -> Primitives.SimulationHeader {
        try Primitives.SimulationHeader(assetId: AssetId(id: assetId), value: value, isUnlimited: isUnlimited)
    }
}

public extension Gemstone.SimulationResult {
    func map() throws -> Primitives.SimulationResult {
        try Primitives.SimulationResult(
            warnings: warnings.map { try $0.map() },
            balanceChanges: balanceChanges.map { try $0.map() },
            payload: payload.map { $0.map() },
            header: header.map { try $0.map() },
        )
    }
}
