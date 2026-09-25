package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.domains.asset.icon
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockFormattedNumber
import com.gemwallet.android.testkit.mockGemConfirmSimulationState
import com.wallet.core.primitives.Chain
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemNumberNotation
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemPrecision
import uniffi.gemstone.GemSimulationBalanceChange
import uniffi.gemstone.GemValueTone
import java.util.Locale

class SimulationTest {

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
                GemSimulationBalanceChange(asset = solana.toGem(), icon = solana.id.icon(), amount = fullAmount(-0.100005, "SOL", GemValueTone.NEGATIVE, "0.100005")),
                GemSimulationBalanceChange(asset = usdc.toGem(), icon = usdc.id.icon(), amount = fullAmount(0.75, "USDC", GemValueTone.POSITIVE, "0.75")),
            ),
        ).toSimulation(mockk(relaxed = true))

        assertEquals(
            listOf("-0.100005 SOL", "+0.75 USDC"),
            simulation.balanceChanges.map { it.listItem().subtitle },
        )
        assertEquals(Chain.Ethereum, simulation.chain)
    }

    private fun fullAmount(value: Double, symbol: String, tone: GemValueTone, exact: String) = mockFormattedNumber(
        value = value,
        unit = GemNumberUnit.Symbol(symbol),
        tone = tone,
        precision = GemPrecision.Fraction(min = 0u, max = 32u),
        notation = GemNumberNotation.SIGNED,
        exact = exact,
    )
}
