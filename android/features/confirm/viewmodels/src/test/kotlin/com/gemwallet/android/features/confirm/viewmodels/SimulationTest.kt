package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import java.util.Locale
import uniffi.gemstone.GemConfirmSimulation
import uniffi.gemstone.GemConfirmSimulationState
import uniffi.gemstone.GemConfirmSession
import uniffi.gemstone.GemSimulationBalanceChange
import com.wallet.core.primitives.Chain
import java.math.BigInteger

class SimulationTest {

    private val confirmSession = mockk<GemConfirmSession>()

    @Before
    fun setUp() {
        Locale.setDefault(Locale.US)
    }

    @Test
    fun `balance changes keep their sign and asset`() {
        val solana = mockAssetSolana()
        val usdc = mockAssetSolanaUSDC()
        val simulation = state(
            balanceChanges = listOf(
                GemSimulationBalanceChange(asset = solana.toGem(), value = BigInteger("-100005000")),
                GemSimulationBalanceChange(asset = usdc.toGem(), value = BigInteger("750000")),
            ),
        ).toSimulation(confirmSession)

        assertEquals(
            listOf("-0.100005 SOL", "+0.75 USDC"),
            simulation.balanceChanges.map { it.formattedValue() },
        )
        assertEquals(
            listOf(ValueDirection.Down, ValueDirection.Up),
            simulation.balanceChanges.map { it.valueDirection() },
        )
    }

    private fun state(
        balanceChanges: List<GemSimulationBalanceChange> = emptyList(),
    ) = GemConfirmSimulationState(
        chain = Chain.Ethereum.string,
        result = null,
        warnings = emptyList(),
        simulation = GemConfirmSimulation(primaryFields = emptyList(), secondaryFields = emptyList(), header = null, balanceChanges = balanceChanges, hasCriticalWarning = false),
        addressNames = emptyList(),
    )
}
