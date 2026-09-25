package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockAssetPriceValue
import com.gemwallet.android.testkit.mockSwapQuote
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemSwapProviderRow
import uniffi.gemstone.SwapProvider
import uniffi.gemstone.swapProviderRow
import uniffi.gemstone.swapQuoteSummary
import java.math.BigInteger

class SwapDetailsUIModelFactoryTest {

    private val payAsset = mockAssetPriceValue(asset = mockAsset(symbol = "AAA", name = "AAA", decimals = 18), price = mockAssetPriceInfo(price = 1.0))
    private val receiveAsset = mockAssetPriceValue(asset = mockAsset(symbol = "BBB", name = "BBB", decimals = 18), price = mockAssetPriceInfo(price = 1.0))

    @Test
    fun `low price impact stays in details and is hidden in summary`() {
        val result = swapDetails(
            toValue = "990000000000000000",
        )

        assertEquals("-1.00%", result!!.priceImpact!!.value.text())
        assertNull(result.summaryPriceImpactText)
        assertNull(result.summaryPriceImpactBadgeText)
    }

    @Test
    fun `medium price impact is shown in summary and warning follows provider mode`() {
        val provider = provider(toValue = "950000000000000000")
        val result = swapDetails(
            toValue = "950000000000000000",
            provider = provider,
            providers = listOf(provider),
            isProviderSelectable = true,
        )

        assertEquals("-5.00%", result!!.summaryPriceImpactText)
        assertEquals("(-5.00%)", result.summaryPriceImpactBadgeText)
        assertFalse(result.shouldShowPriceImpactWarning)
        assertTrue(result.isProviderSelectable)
    }

    @Test
    fun `price impact uses shared ios rounding behavior`() {
        val result = swapDetails(
            toValue = "1023400000000000000",
        )

        assertEquals("+2.34%", result!!.priceImpact!!.value.text())
        assertNull("a positive impact warns nobody", result.priceImpact.warning)
    }

    @Test
    fun `rate uses ios precision for tiny swap values`() {
        val result = swapDetails(toValue = "20446939000000")

        assertEquals("1 AAA ≈ 0.00002045 BBB", result!!.rate.forward)
    }

    @Test
    fun `rate handles cross decimal assets`() {
        val eth = mockAssetPriceValue(asset = mockAssetEthereum(), price = mockAssetPriceInfo(price = 1.0))
        val usdc = mockAssetPriceValue(asset = mockAsset(symbol = "USDC", name = "USDC", decimals = 6), price = mockAssetPriceInfo(price = 1.0))
        val result = SwapDetailsUIModelFactory.create(
            SwapDetailsUIModelInput(
                summary = summary("1000000000000000000", "2000000000", DEFAULT_SLIPPAGE_BPS, eth, usdc),
                provider = provider(
                    toValue = "2000000000",
                    receiveAsset = usdc,
                ),
                slippageBps = DEFAULT_SLIPPAGE_BPS,
                selectedSlippage = DEFAULT_SLIPPAGE_BPS,
                isProviderSelectable = false,
            ),
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
            ),
        )
    }

    @Test
    fun `returns null when receive amount is zero`() {
        assertNull(swapDetails(toValue = "0"))
    }

    private fun swapDetails(
        fromValue: String = DEFAULT_FROM_VALUE,
        toValue: String,
        provider: GemSwapProviderRow = provider(toValue),
        providers: List<GemSwapProviderRow> = emptyList(),
        slippageBps: UInt = DEFAULT_SLIPPAGE_BPS,
        isProviderSelectable: Boolean = false,
    ): SwapDetailsUIModel? {
        val summary = summary(fromValue, toValue, slippageBps, payAsset, receiveAsset)
        return SwapDetailsUIModelFactory.create(
            SwapDetailsUIModelInput(
                summary = summary,
                provider = provider,
                providers = providers,
                slippageBps = slippageBps,
                selectedSlippage = slippageBps,
                isProviderSelectable = isProviderSelectable,
            ),
        )
    }

    private fun summary(fromValue: String, toValue: String, slippageBps: UInt, payAsset: AssetPriceValue, receiveAsset: AssetPriceValue) = swapQuoteSummary(
        mockSwapQuote(fromAmount = fromValue.toBigInteger(), toAmount = toValue.toBigInteger(), slippageBps = slippageBps),
        payAsset.asset.toGem(),
        receiveAsset.asset.toGem(),
        payAsset.price?.price?.price,
        receiveAsset.price?.price?.price,
    )

    private fun provider(toValue: String, receiveAsset: AssetPriceValue = this.receiveAsset) = swapProviderRow(
        provider = SwapProvider.OKX,
        title = "OKX (DEX)",
        toValue = BigInteger(toValue),
        receiveAsset = receiveAsset.asset.toGem(),
        receivePrice = receiveAsset.price?.price?.price,
        currency = Currency.USD.toGem(),
        isSelected = true,
    )

    private companion object {
        const val DEFAULT_FROM_VALUE = "1000000000000000000"
        const val DEFAULT_TO_VALUE = "1000000000000000000"
        val DEFAULT_SLIPPAGE_BPS = 100u
    }
}
