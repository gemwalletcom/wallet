package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import java.util.Locale
import uniffi.gemstone.GemApprovalValue
import uniffi.gemstone.GemConfirmSimulation
import uniffi.gemstone.GemConfirmSimulationState
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemSimulationBalanceChange
import uniffi.gemstone.GemSimulationValue

class SimulationTest {

    private val confirmService = mockk<GemConfirmTransferService>()

    @Before
    fun setUp() {
        Locale.setDefault(Locale.US)
    }

    @Test
    fun `a balance change with an unreadable value is dropped`() {
        val solana = mockAssetSolana()
        val usdc = mockAssetSolanaUSDC()
        val simulation = state(
            balanceChanges = listOf(
                GemSimulationBalanceChange(asset = solana.toGem(), value = "-100005000"),
                GemSimulationBalanceChange(asset = usdc.toGem(), value = "750000"),
                GemSimulationBalanceChange(asset = usdc.toGem(), value = ""),
            ),
        ).toSimulation(warnings = emptyList(), chain = null, confirmService = confirmService)

        assertEquals(
            listOf("-0.100005 SOL", "+0.75 USDC"),
            simulation.balanceChanges.map { it.formattedValue() },
        )
        assertEquals(
            listOf(ValueDirection.Down, ValueDirection.Up),
            simulation.balanceChanges.map { it.valueDirection() },
        )
    }

    @Test
    fun `an unlimited header carries no amount`() {
        val usdc = mockAssetSolanaUSDC()
        val unlimited = state(header = GemSimulationValue(asset = usdc.toGem(), value = GemApprovalValue.Unlimited))
            .toSimulation(warnings = emptyList(), chain = null, confirmService = confirmService)

        assertTrue(unlimited.headerIsUnlimited)
        assertNull(unlimited.headerValue)
        assertEquals(usdc, unlimited.headerAsset)

        val exact = state(header = GemSimulationValue(asset = usdc.toGem(), value = GemApprovalValue.Exact("750000")))
            .toSimulation(warnings = emptyList(), chain = null, confirmService = confirmService)

        assertEquals(false, exact.headerIsUnlimited)
        assertEquals("750000", exact.headerValue)
    }

    private fun state(
        balanceChanges: List<GemSimulationBalanceChange> = emptyList(),
        header: GemSimulationValue? = null,
    ) = GemConfirmSimulationState(
        simulation = GemConfirmSimulation(primaryFields = emptyList(), secondaryFields = emptyList(), header = header, balanceChanges = balanceChanges),
        addressNames = emptyList(),
    )
}
