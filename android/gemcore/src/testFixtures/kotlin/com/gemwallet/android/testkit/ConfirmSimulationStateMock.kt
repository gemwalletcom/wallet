package com.gemwallet.android.testkit

import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemConfirmSimulation
import uniffi.gemstone.GemConfirmSimulationState
import uniffi.gemstone.GemSimulationBalanceChange

fun mockGemConfirmSimulationState(
    chain: Chain = Chain.Ethereum,
    balanceChanges: List<GemSimulationBalanceChange>? = null,
) = GemConfirmSimulationState(
    chain = chain.string,
    result = null,
    warnings = emptyList(),
    simulation = balanceChanges?.let {
        GemConfirmSimulation(
            primaryFields = emptyList(),
            secondaryFields = emptyList(),
            header = null,
            balanceChanges = it,
            hasCriticalWarning = false,
        )
    },
)
