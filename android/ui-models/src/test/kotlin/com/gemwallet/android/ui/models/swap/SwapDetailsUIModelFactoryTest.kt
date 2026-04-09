package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.format
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.SwapPriceImpact
import uniffi.gemstone.SwapPriceImpactType
import uniffi.gemstone.SwapperProvider

class SwapDetailsUIModelFactoryTest {

    private val payAsset = assetInfo(symbol = "AAA")
    private val receiveAsset = assetInfo(symbol = "BBB")

    @Test
    fun `low price impact stays in details and is hidden in summary`() {
        val result = swapDetails(
            toValue = "990000000000000000",
            etaInSeconds = 30u,
            priceImpact = SwapPriceImpact(
                percentage = -1.0,
                impactType = SwapPriceImpactType.LOW,
                isHigh = false,
            ),
        )

        assertEquals("-1.00%", result!!.priceImpact!!.displayText)
        assertNull(result.summaryPriceImpactText)
        assertNull(result.summaryPriceImpactBadgeText)
        assertEquals("1.00%", result.slippageText)
        assertNull(result.estimatedTime)
    }

    @Test
    fun `medium price impact is shown in summary and warning follows provider mode`() {
        val provider = provider(toValue = "950000000000000000")
        val result = swapDetails(
            toValue = "950000000000000000",
            provider = provider,
            providers = listOf(provider),
            etaInSeconds = 180u,
            isProviderSelectable = true,
            priceImpact = SwapPriceImpact(
                percentage = -5.0,
                impactType = SwapPriceImpactType.MEDIUM,
                isHigh = false,
            ),
        )

        assertEquals("-5.00%", result!!.summaryPriceImpactText)
        assertEquals("(-5.00%)", result.summaryPriceImpactBadgeText)
        assertFalse(result.shouldShowPriceImpactWarning)
        assertTrue(result.isProviderSelectable)
        assertEquals("≈ 3 min", result.estimatedTime)
    }

    @Test
    fun `price impact uses shared ios rounding behavior`() {
        val result = swapDetails(
            toValue = DEFAULT_TO_VALUE,
            priceImpact = SwapPriceImpact(
                percentage = 2.345,
                impactType = SwapPriceImpactType.POSITIVE,
                isHigh = false,
            ),
        )

        assertEquals("+2.34%", result!!.priceImpact!!.displayText)
        assertEquals("2.34%", result.priceImpact.warningText)
    }

    @Test
    fun `estimated time matches ios minute truncation`() {
        assertNull(swapDetails(toValue = "950000000000000000", etaInSeconds = 60u)!!.estimatedTime)
        assertEquals("≈ 1 min", swapDetails(toValue = "950000000000000000", etaInSeconds = 61u)!!.estimatedTime)
        assertEquals("≈ 1 min", swapDetails(toValue = "950000000000000000", etaInSeconds = 119u)!!.estimatedTime)
        assertEquals("≈ 2 min", swapDetails(toValue = "950000000000000000", etaInSeconds = 120u)!!.estimatedTime)
    }

    @Test
    fun `rate uses ios precision for tiny swap values`() {
        val result = swapDetails(toValue = "20446939000000")

        assertEquals("1 AAA ≈ 0.00002045 BBB", result!!.rate.forward)
    }

    @Test
    fun `rate handles cross decimal assets`() {
        val result = SwapDetailsUIModelFactory.create(
            SwapDetailsUIModelInput(
                payAsset = assetInfo(symbol = "ETH", decimals = 18),
                receiveAsset = assetInfo(symbol = "USDC", decimals = 6),
                fromValue = "1000000000000000000",
                toValue = "2000000000",
                provider = provider(
                    toValue = "2000000000",
                    receiveAsset = assetInfo(symbol = "USDC", decimals = 6),
                ),
                slippageBps = DEFAULT_SLIPPAGE_BPS,
                etaInSeconds = null,
                isProviderSelectable = false,
            ),
            priceImpactCalculator = { _, _ -> null },
        )

        assertEquals("1 ETH ≈ 2,000.00 USDC", result!!.rate.forward)
        assertEquals("1 USDC ≈ 0.0005 ETH", result.rate.reverse)
    }

    @Test
    fun `returns null when pay amount is zero`() {
        assertNull(
            swapDetails(
                fromValue = "0",
                toValue = "950000000000000000",
            )
        )
    }

    @Test
    fun `returns null when receive amount is zero`() {
        assertNull(swapDetails(toValue = "0"))
    }

    @Test
    fun `returns null when amount input is invalid`() {
        assertNull(
            swapDetails(
                fromValue = "invalid",
                toValue = "950000000000000000",
            )
        )
    }

    @Test
    fun `minimum receive matches output when slippage is zero`() {
        val result = swapDetails(
            toValue = DEFAULT_TO_VALUE,
            slippageBps = 0u,
        )

        assertEquals(formattedReceiveAmount(DEFAULT_TO_VALUE), result!!.minimumReceive)
    }

    @Test
    fun `minimum receive floors to zero for a single atomic unit`() {
        val result = swapDetails(
            toValue = "1",
            slippageBps = DEFAULT_SLIPPAGE_BPS,
        )

        assertEquals(formattedReceiveAmount("0"), result!!.minimumReceive)
    }

    private fun swapDetails(
        fromValue: String = DEFAULT_FROM_VALUE,
        toValue: String,
        provider: SwapProviderUIModel = provider(toValue),
        providers: List<SwapProviderUIModel> = emptyList(),
        slippageBps: UInt = DEFAULT_SLIPPAGE_BPS,
        etaInSeconds: UInt? = null,
        isProviderSelectable: Boolean = false,
        priceImpact: SwapPriceImpact? = null,
    ) = SwapDetailsUIModelFactory.create(
        SwapDetailsUIModelInput(
            payAsset = payAsset,
            receiveAsset = receiveAsset,
            fromValue = fromValue,
            toValue = toValue,
            provider = provider,
            providers = providers,
            slippageBps = slippageBps,
            etaInSeconds = etaInSeconds,
            isProviderSelectable = isProviderSelectable,
        ),
        priceImpactCalculator = { _, _ -> priceImpact },
    )

    private fun provider(
        toValue: String,
        receiveAsset: com.gemwallet.android.model.AssetInfo = this.receiveAsset,
    ) = SwapProviderUIModelFactory.create(
        providerId = SwapperProvider.OKX,
        title = "OKX (DEX)",
        receiveAsset = receiveAsset,
        toValue = toValue,
    )

    private fun assetInfo(
        symbol: String,
        decimals: Int = 18,
    ) = mockAssetInfo(
        asset = mockAsset(symbol = symbol, name = symbol, decimals = decimals),
    ).copy(
        price = mockAssetPriceInfo(price = 1.0),
    )

    private fun formattedReceiveAmount(atomicValue: String) =
        receiveAsset.asset.format(Crypto(atomicValue), 2, dynamicPlace = true)

    private companion object {
        const val DEFAULT_FROM_VALUE = "1000000000000000000"
        const val DEFAULT_TO_VALUE = "1000000000000000000"
        val DEFAULT_SLIPPAGE_BPS = 100u
    }
}
