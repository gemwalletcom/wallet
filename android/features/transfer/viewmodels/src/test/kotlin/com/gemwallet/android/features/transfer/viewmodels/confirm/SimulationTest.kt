package com.gemwallet.android.features.transfer.viewmodels.confirm

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockGemFormattedNumber
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemNumberDisplay
import uniffi.gemstone.GemNumberNotation
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemPrecision
import uniffi.gemstone.GemSimulationBalanceChange
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.assetText
import java.util.Locale

class SimulationTest {

    @Before
    fun setUp() {
        Locale.setDefault(Locale.US)
    }

    @Test
    fun `balance changes keep their sign and asset`() {
        val solana = mockAsset(id = mockAssetId(chain = Chain.Solana), name = "Solana", symbol = "SOL", decimals = 9)
        val usdc = mockAsset(id = mockAssetId(chain = Chain.Solana, tokenId = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), name = "USD Coin", symbol = "USDC", decimals = 6, type = AssetType.SPL)
        val changes = listOf(
            GemSimulationBalanceChange(asset = solana.toGem(), icon = assetText(solana.toGem()).icon, amount = fullAmount(-0.100005, "SOL", GemValueTone.NEGATIVE, "0.100005")),
            GemSimulationBalanceChange(asset = usdc.toGem(), icon = assetText(usdc.toGem()).icon, amount = fullAmount(0.75, "USDC", GemValueTone.POSITIVE, "0.75")),
        )

        assertEquals(listOf("-0.100005 SOL", "+0.75 USDC"), changes.map { it.listItem().subtitle })
    }

    private fun fullAmount(value: Double, symbol: String, tone: GemValueTone, exact: String) = mockGemFormattedNumber(
        value = value,
        unit = GemNumberUnit.Symbol(symbol),
        display = GemNumberDisplay.Number(precision = GemPrecision.Fraction(min = 0u, max = 32u)),
        notation = GemNumberNotation.SIGNED,
        tone = tone,
        exact = exact,
    )
}
