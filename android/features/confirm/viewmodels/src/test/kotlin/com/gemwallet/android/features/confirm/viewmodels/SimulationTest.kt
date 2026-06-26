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
        val unknownAssetId = AssetId(Chain.Solana, "UnknownMint11111111111111111111111111111111")
        val simulation = SimulationResult(
            warnings = emptyList(),
            balanceChanges = listOf(
                SimulationBalanceChange(assetId = solana.id, value = "-100005000", decimals = 9, name = solana.name, symbol = solana.symbol),
                SimulationBalanceChange(assetId = usdc.id, value = "750000", decimals = 6, name = usdc.name, symbol = usdc.symbol),
                SimulationBalanceChange(assetId = unknownAssetId, value = "-42", decimals = 2, name = null, symbol = null),
            ),
            payload = emptyList(),
            header = null,
        ).toSimulation()

        assertEquals(
            listOf("-0.100005 SOL", "+0.75 USDC", "-0.42"),
            simulation.balanceChanges.map { it.formattedValue() },
        )
        assertEquals(
            listOf(ValueDirection.Down, ValueDirection.Up, ValueDirection.Down),
            simulation.balanceChanges.map { it.valueDirection() },
        )
        assertEquals(
            listOf(false, false, true),
            simulation.balanceChanges.map { it.isUnknown },
        )
    }
}
