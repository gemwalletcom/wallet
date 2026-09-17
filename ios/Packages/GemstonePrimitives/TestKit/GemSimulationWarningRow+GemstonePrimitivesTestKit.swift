// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSimulationWarningKind
import struct Gemstone.GemSimulationWarningRow
import enum Gemstone.GemSimulationWarningTitle
import enum Gemstone.SimulationSeverity

public extension GemSimulationWarningRow {
    static func mock(
        kind: GemSimulationWarningKind = .unlimitedApproval,
        severity: SimulationSeverity = .warning,
        message: String? = nil,
        title: GemSimulationWarningTitle? = nil,
    ) -> GemSimulationWarningRow {
        GemSimulationWarningRow(kind: kind, title: title ?? Self.title(kind: kind, severity: severity), severity: severity, message: message)
    }

    private static func title(kind: GemSimulationWarningKind, severity: SimulationSeverity) -> GemSimulationWarningTitle {
        switch kind {
        case .unlimitedApproval: .unlimitedApproval
        case .nftCollectionApproval: .nftCollectionApproval
        case .externallyOwnedSpender: .warning
        case .suspiciousSpender: .error
        case .validationError: severity == .critical ? .error : .warning
        }
    }
}
