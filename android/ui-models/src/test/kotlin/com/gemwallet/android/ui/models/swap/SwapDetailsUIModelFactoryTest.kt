package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.ValueFormatter
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
import uniffi.gemstone.GemValueStyle
import uniffi.gemstone.SwapPriceImpact
import uniffi.gemstone.SwapPriceImpactType
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
            etaInSeconds = 30u,
            priceImpact = SwapPriceImpact(
                percentage = -1.0,
                impactType = SwapPriceImpactType.LOW,
                isHigh = false,
                showsInSummary = false,
            ),
        )

        assertEquals("-1.00%", result!!.priceImpact!!.displayText)
        assertNull(result.summaryPriceImpactText)
        assertNull(result.summaryPriceImpactBadgeText)
        assertEquals("1.00%", result.slippageText)
        assertEquals(30u, result.etaInSeconds)
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
                showsInSummary = true,
            ),
        )

        assertEquals("-5.00%", result!!.summaryPriceImpactText)
        assertEquals("(-5.00%)", result.summaryPriceImpactBadgeText)
        assertFalse(result.shouldShowPriceImpactWarning)
        assertTrue(result.isProviderSelectable)
        assertEquals(180u, result.etaInSeconds)
    }

    @Test
    fun `price impact uses shared ios rounding behavior`() {
        val result = swapDetails(
            toValue = DEFAULT_TO_VALUE,
            priceImpact = SwapPriceImpact(
                percentage = 2.345,
                impactType = SwapPriceImpactType.POSITIVE,
                isHigh = false,
                showsInSummary = false,
            ),
        )

        assertEquals("+2.34%", result!!.priceImpact!!.displayText)
        assertEquals("2.34%", result.priceImpact.warningText)
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
                payAsset = eth,
                receiveAsset = usdc,
                summary = summary("1000000000000000000", "2000000000", DEFAULT_SLIPPAGE_BPS, null, eth, usdc),
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

    @Test
    fun `minimum receive matches output when slippage is zero`() {
        val result = swapDetails(
            toValue = DEFAULT_TO_VALUE,
            slippageBps = 0u,
        )

        assertEquals(formattedReceiveAmount(DEFAULT_TO_VALUE), result!!.minimumReceive)
    }

    @Test
    fun `minimum receive applies sub percent slippage`() {
        val result = swapDetails(
            toValue = DEFAULT_TO_VALUE,
            slippageBps = 50u,
        )

        assertEquals(formattedReceiveAmount("995000000000000000"), result!!.minimumReceive)
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
        provider: GemSwapProviderRow = provider(toValue),
        providers: List<GemSwapProviderRow> = emptyList(),
        slippageBps: UInt = DEFAULT_SLIPPAGE_BPS,
        etaInSeconds: UInt? = null,
        isProviderSelectable: Boolean = false,
        priceImpact: SwapPriceImpact? = null,
    ): SwapDetailsUIModel? {
        val summary = summary(fromValue, toValue, slippageBps, etaInSeconds, payAsset, receiveAsset)
        return SwapDetailsUIModelFactory.create(
            SwapDetailsUIModelInput(
                payAsset = payAsset,
                receiveAsset = receiveAsset,
                summary = summary,
                provider = provider,
                providers = providers,
                slippageBps = slippageBps,
                selectedSlippage = slippageBps,
                isProviderSelectable = isProviderSelectable,
                priceImpact = priceImpact,
            ),
        )
    }

    private fun summary(fromValue: String, toValue: String, slippageBps: UInt, etaInSeconds: UInt?, payAsset: AssetPriceValue, receiveAsset: AssetPriceValue) = swapQuoteSummary(
        mockSwapQuote(fromAmount = fromValue.toBigInteger(), toAmount = toValue.toBigInteger(), slippageBps = slippageBps, etaInSeconds = etaInSeconds),
        payAsset.asset.toGem(),
        receiveAsset.asset.toGem(),
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

    private fun formattedReceiveAmount(atomicValue: String) = ValueFormatter(style = GemValueStyle.AUTO)
        .string(java.math.BigInteger(atomicValue), receiveAsset.asset)

    private companion object {
        const val DEFAULT_FROM_VALUE = "1000000000000000000"
        const val DEFAULT_TO_VALUE = "1000000000000000000"
        val DEFAULT_SLIPPAGE_BPS = 100u
    }
}
