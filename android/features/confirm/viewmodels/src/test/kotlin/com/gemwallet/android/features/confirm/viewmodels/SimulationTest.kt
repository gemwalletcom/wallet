package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.domains.asset.icon
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockGemConfirmSimulationState
import com.wallet.core.primitives.Chain
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemAmountSign
import uniffi.gemstone.GemSimulationBalanceChange
import uniffi.gemstone.GemValueTone
import java.math.BigInteger
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
                GemSimulationBalanceChange(asset = solana.toGem(), icon = solana.id.icon(), value = BigInteger("-100005000"), sign = GemAmountSign.OUTGOING, tone = GemValueTone.NEGATIVE),
                GemSimulationBalanceChange(asset = usdc.toGem(), icon = usdc.id.icon(), value = BigInteger("750000"), sign = GemAmountSign.INCOMING, tone = GemValueTone.POSITIVE),
            ),
        ).toSimulation(mockk(relaxed = true))

        assertEquals(
            listOf("-0.100005 SOL", "+0.75 USDC"),
            simulation.balanceChanges.map { it.formattedValue() },
        )
        assertEquals(Chain.Ethereum, simulation.chain)
    }
}
