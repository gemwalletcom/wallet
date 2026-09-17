// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSimulationWarningKind
import struct Gemstone.GemSimulationWarningRow
import enum Gemstone.SimulationSeverity

public extension GemSimulationWarningRow {
    static func mock(
        kind: GemSimulationWarningKind = .unlimitedApproval,
        severity: SimulationSeverity = .warning,
        message: String? = nil,
    ) -> GemSimulationWarningRow {
        GemSimulationWarningRow(kind: kind, severity: severity, message: message)
    }
}
