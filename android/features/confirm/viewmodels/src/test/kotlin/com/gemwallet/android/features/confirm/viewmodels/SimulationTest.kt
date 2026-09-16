package com.gemwallet.android.features.confirm.viewmodels

import uniffi.gemstone.GemValueTone
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockGemConfirmSimulationState
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import java.util.Locale
import uniffi.gemstone.GemAmountSign
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemSimulationBalanceChange
import java.math.BigInteger

class SimulationTest {

    private val confirmation = mockk<GemConfirmation>()

    @Before
    fun setUp() {
        Locale.setDefault(Locale.US)
    }

    @Test
    fun `balance changes keep their sign and asset`() {
        val solana = mockAssetSolana()
        val usdc = mockAssetSolanaUSDC()
        val simulation = mockGemConfirmSimulationState(
            balanceChanges = listOf(
                GemSimulationBalanceChange(asset = solana.toGem(), value = BigInteger("-100005000"), sign = GemAmountSign.OUTGOING),
                GemSimulationBalanceChange(asset = usdc.toGem(), value = BigInteger("750000"), sign = GemAmountSign.INCOMING),
            ),
        ).toSimulation(confirmation)

        assertEquals(
            listOf("-0.100005 SOL", "+0.75 USDC"),
            simulation.balanceChanges.map { it.formattedValue() },
        )
        assertEquals(
            listOf(GemValueTone.NEGATIVE, GemValueTone.POSITIVE),
            simulation.balanceChanges.map { it.tone() },
        )
    }
}
