package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationBalanceChange
import com.wallet.core.primitives.SimulationResult
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import java.util.Locale

class SimulationTest {
    @Before
    fun setUp() {
        Locale.setDefault(Locale.US)
    }

    @Test
    fun balanceChanges_formatSignedAssetDeltasWithDirection() {
        val solana = mockAssetSolana()
        val usdc = mockAssetSolanaUSDC()
        val missingAssetId = AssetId(Chain.Solana, "MissingMint111111111111111111111111111111111")
        val simulation = SimulationResult(
            warnings = emptyList(),
            balanceChanges = listOf(
                SimulationBalanceChange(assetId = solana.id, value = "-100005000"),
                SimulationBalanceChange(assetId = usdc.id, value = "750000"),
                SimulationBalanceChange(assetId = missingAssetId, value = "-42"),
            ),
            payload = emptyList(),
            header = null,
        ).toSimulation(balanceChangeAssets = listOf(solana, usdc))

        assertEquals(
            listOf("-0.100005 SOL", "+0.75 USDC", "-42 Missin...111111"),
            simulation.balanceChanges.map { it.formattedValue() },
        )
        assertEquals(
            listOf(ValueDirection.Down, ValueDirection.Up, ValueDirection.Down),
            simulation.balanceChanges.map { it.valueDirection() },
        )
    }
}
