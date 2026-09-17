package com.gemwallet.android.testkit

import uniffi.gemstone.GemSimulationWarningKind
import uniffi.gemstone.GemSimulationWarningRow
import uniffi.gemstone.SimulationSeverity

fun mockGemSimulationWarningRow(
    kind: GemSimulationWarningKind,
    severity: SimulationSeverity = SimulationSeverity.WARNING,
    message: String? = null,
) = GemSimulationWarningRow(
    kind = kind,
    severity = severity,
    message = message,
)
