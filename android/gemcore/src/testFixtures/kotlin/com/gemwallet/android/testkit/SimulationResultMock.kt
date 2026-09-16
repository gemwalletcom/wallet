package com.gemwallet.android.testkit

import uniffi.gemstone.SimulationResult

fun mockSimulationResult() = SimulationResult(
    warnings = emptyList(),
    balanceChanges = emptyList(),
    payload = emptyList(),
    header = null,
)
