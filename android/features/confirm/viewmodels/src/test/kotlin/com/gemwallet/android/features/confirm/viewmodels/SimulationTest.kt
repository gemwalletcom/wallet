package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
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
    fun balanceChanges_skipUnresolvedAssets() {
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
        ).toSimulation(assets = listOf(solana, usdc).associateBy { it.id })

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
    fun balanceChanges_useResolvedAssetMetadata() {
        val dust = mockAsset(
            chain = Chain.Ton,
            tokenId = "EQBlqsm144Dq6SjbPI4jjZvA1hqTIP3CvHovbIfW_t-SCALE",
            name = "DeDust",
            symbol = "DUST",
            decimals = 9,
            type = AssetType.JETTON,
        )
        val simulation = SimulationResult(
            warnings = emptyList(),
            balanceChanges = listOf(
                SimulationBalanceChange(assetId = dust.id, value = "2244508455", decimals = 0, name = null, symbol = null),
            ),
            payload = emptyList(),
            header = null,
        ).toSimulation(assets = mapOf(dust.id to dust))

        assertEquals("+2.244508455 DUST", simulation.balanceChanges.single().formattedValue())
        assertEquals(dust, simulation.balanceChanges.single().asset)
    }
}
