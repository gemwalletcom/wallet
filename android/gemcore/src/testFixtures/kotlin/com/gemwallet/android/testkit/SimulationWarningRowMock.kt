package com.gemwallet.android.testkit

import uniffi.gemstone.GemSimulationWarningKind
import uniffi.gemstone.GemSimulationWarningRow
import uniffi.gemstone.GemSimulationWarningTitle
import uniffi.gemstone.SimulationSeverity

fun mockGemSimulationWarningRow(
    kind: GemSimulationWarningKind,
    severity: SimulationSeverity = SimulationSeverity.WARNING,
    message: String? = null,
    title: GemSimulationWarningTitle = when (kind) {
        GemSimulationWarningKind.UNLIMITED_APPROVAL -> GemSimulationWarningTitle.UNLIMITED_APPROVAL
        GemSimulationWarningKind.NFT_COLLECTION_APPROVAL -> GemSimulationWarningTitle.NFT_COLLECTION_APPROVAL
        GemSimulationWarningKind.EXTERNALLY_OWNED_SPENDER -> GemSimulationWarningTitle.WARNING
        GemSimulationWarningKind.SUSPICIOUS_SPENDER -> GemSimulationWarningTitle.ERROR
        GemSimulationWarningKind.VALIDATION_ERROR -> if (severity == SimulationSeverity.CRITICAL) GemSimulationWarningTitle.ERROR else GemSimulationWarningTitle.WARNING
    },
) = GemSimulationWarningRow(
    kind = kind,
    title = title,
    severity = severity,
    message = message,
)
