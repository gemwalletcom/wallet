// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import enum Gemstone.SimulationSeverity
import struct Gemstone.SimulationWarning
import struct Gemstone.SimulationWarningApproval
import enum Gemstone.SimulationWarningType
import Primitives

public extension SimulationWarning {
    static func mock(
        severity: SimulationSeverity = .warning,
        warning: SimulationWarningType = .tokenApproval(.mock()),
        message: String? = nil,
    ) -> SimulationWarning {
        SimulationWarning(
            severity: severity,
            warning: warning,
            message: message,
        )
    }
}

public extension SimulationWarningApproval {
    static func mock(
        assetId: Primitives.AssetId = Primitives.AssetId(chain: .ethereum, tokenId: "0x123"),
        value: BigInt? = nil,
    ) -> SimulationWarningApproval {
        SimulationWarningApproval(
            assetId: assetId.identifier,
            value: value,
        )
    }
}
