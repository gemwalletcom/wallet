package com.gemwallet.android.domains.asset.aggregates

import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetMetaData
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockGemAssetRowStyle
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemAssetBalanceScope
import uniffi.gemstone.GemAssetSubtitleStyle
import uniffi.gemstone.GemAssetTitleStyle
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemValueTone
import java.math.BigInteger

class AssetInfoDataAggregateTest {

    private val btcAsset = mockAsset()

    private val ethAsset = mockAssetEthereum()

    private val account = mockAccount(address = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh")

    @Test
    fun assetInfoDataAggregate_id_returnsAssetId() {
        val assetInfo = mockAssetInfo(asset = btcAsset)
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertEquals(btcAsset.id, aggregate.id)
    }

    @Test
    fun assetInfoDataAggregate_title_returnsAssetName() {
        val assetInfo = mockAssetInfo(asset = btcAsset)
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertEquals("Bitcoin", aggregate.title)
    }

    @Test
    fun assetInfoDataAggregate_titleSymbolAndNetwork_areTheOnesCoreResolved() {
        val usdc = mockAsset(chain = Chain.Ethereum, tokenId = "0xusdc", name = "USDC", symbol = "USDC")
        val assetInfo = mockAssetInfo(asset = usdc)
        val networked = assetInfo.toAssetInfoDataAggregate(
            mockGemAssetRowStyle(title = GemAssetTitleStyle.NETWORK, showsSymbol = true, subtitle = GemAssetSubtitleStyle.NETWORK),
        )

        assertEquals("Ethereum", networked.title)
        assertEquals("USDC", networked.symbol)
        assertEquals("Ethereum", networked.network)

        val named = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(showsSymbol = true))

        assertEquals("USDC", named.title)
        assertNull("a symbol that repeats the title is not shown", named.symbol)
        assertNull(named.network)
    }

    @Test
    fun assetInfoDataAggregate_asset_returnsAsset() {
        val assetInfo = mockAssetInfo(asset = btcAsset)
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertEquals(btcAsset, aggregate.asset)
    }

    @Test
    fun assetInfoDataAggregate_balance_hideBalanceTrue_returnsStars() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("100000000")),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = true)

        assertEquals("*****", aggregate.balance)
    }

    @Test
    fun assetInfoDataAggregate_balance_hideBalanceFalse_returnsFormattedBalance() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("100000000")),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)
        assertEquals("1 BTC", aggregate.balance)
    }

    @Test
    fun assetInfoDataAggregate_balance_zeroBalance_returnsFormattedZero() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("0")),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertEquals("0 BTC", aggregate.balance)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_hideBalanceTrue_returnsStars() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("100000000")),
            price = mockAssetPriceInfo(price = 50000.0),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = true)

        assertEquals("*****", aggregate.balanceEquivalent)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_withPrice_returnsFormattedFiat() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("100000000")),
            price = mockAssetPriceInfo(price = 50000.0),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)
        assertEquals("\$50,000.00", aggregate.balanceEquivalent)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_noPrice_returnsEmpty() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("100000000")),
            price = null,
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertEquals("", aggregate.balanceEquivalent)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_zeroPriceValue_returnsEmpty() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("100000000")),
            price = mockAssetPriceInfo(price = 0.0),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertEquals("", aggregate.balanceEquivalent)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_nonFinitePrice_returnsEmpty() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("100000000")),
            price = mockAssetPriceInfo(price = Double.NaN),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertEquals("", aggregate.balanceEquivalent)
        assertNull("a price nobody quoted is no price", aggregate.price.price)
    }

    @Test
    fun assetInfoDataAggregate_availableScope_showsWhatIsSpendable() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("100000000"), staked = BigInteger("200000000")),
        )

        assertEquals("3 BTC", assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle()).balance)
        assertEquals("1 BTC", assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), scope = GemAssetBalanceScope.AVAILABLE).balance)
    }

    @Test
    fun assetInfoDataAggregate_isZeroBalance_zeroBalance_returnsTrue() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("0")),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertTrue(aggregate.isZeroBalance)
    }

    @Test
    fun assetInfoDataAggregate_price_withPrice_returnsPriceableValue() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = -5.000002),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertEquals(50000.0, aggregate.price.price?.value)
        assertEquals("\$50,000.00", aggregate.price.price?.text())
        assertEquals(-5.000002, aggregate.price.change?.value)
        assertEquals("-5.00%", aggregate.price.change?.text())
        assertEquals(GemValueTone.NEGATIVE, aggregate.price.change?.tone)
    }

    @Test
    fun assetInfoDataAggregate_small_negative_change_value() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = -0.000006),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertEquals(-0.000006, aggregate.price.change?.value)
        assertEquals("-0.00%", aggregate.price.change?.text())
        assertEquals(GemValueTone.NEGATIVE, aggregate.price.change?.tone)
    }

    @Test
    fun assetInfoDataAggregate_small_positive_change_value() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = 0.000006),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertEquals(0.000006, aggregate.price.change?.value)
        assertEquals("+0.00%", aggregate.price.change?.text())
        assertEquals(GemValueTone.POSITIVE, aggregate.price.change?.tone)
    }

    @Test
    fun assetInfoDataAggregate_small_change_value() {
        mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = -0.06),
        ).toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false).let { aggregate ->
            assertEquals("-0.06%", aggregate.price.change?.text())
            assertEquals(GemValueTone.NEGATIVE, aggregate.price.change?.tone)
        }
        mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = -0.02),
        ).toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false).let { aggregate ->
            assertEquals("-0.02%", aggregate.price.change?.text())
            assertEquals(GemValueTone.NEGATIVE, aggregate.price.change?.tone)
        }

        mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = 0.06),
        ).toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false).let { aggregate ->
            assertEquals("+0.06%", aggregate.price.change?.text())
            assertEquals(GemValueTone.POSITIVE, aggregate.price.change?.tone)
        }

        mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = 0.02),
        ).toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false).let { aggregate ->
            assertEquals("+0.02%", aggregate.price.change?.text())
            assertEquals(GemValueTone.POSITIVE, aggregate.price.change?.tone)
        }
    }

    @Test
    fun assetInfoDataAggregate_price_noPrice_returnsNull() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            price = null,
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertNull(aggregate.price.price)
        assertNull(aggregate.price.change)
    }

    @Test
    fun assetInfoDataAggregate_pinned_pinnedTrue_returnsTrue() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            metadata = mockAssetMetaData(isPinned = true),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertTrue(aggregate.pinned)
    }

    @Test
    fun assetInfoDataAggregate_pinned_pinnedFalse_returnsFalse() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            metadata = mockAssetMetaData(isPinned = false),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertFalse(aggregate.pinned)
    }

    @Test
    fun assetInfoDataAggregate_accountAddress_withOwner_returnsAddress() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            owner = account,
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertEquals("bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh", aggregate.accountAddress)
    }

    @Test
    fun assetInfoDataAggregate_accountAddress_noOwner_returnsEmpty() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            owner = null,
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertEquals("", aggregate.accountAddress)
    }

    @Test
    fun assetInfoDataAggregate_price_withEuroCurrency_returnsEuro() {
        val assetInfo = mockAssetInfo(
            asset = ethAsset,
            price = mockAssetPriceInfo(price = 3000.0, currency = Currency.EUR),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertEquals(GemNumberUnit.Currency(code = "EUR"), aggregate.price.price?.unit)
        assertEquals(3000.0, aggregate.price.price?.value)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_withEuroCurrency_returnsEuroFormat() {
        val assetInfo = mockAssetInfo(
            asset = ethAsset,
            balance = AssetBalance.create(ethAsset, available = BigInteger("1000000000000000000")),
            price = mockAssetPriceInfo(price = 3000.0, currency = Currency.EUR),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertEquals("€3,000.00", aggregate.balanceEquivalent)
    }

    @Test
    fun assetInfoDataAggregate_balance_multipleBalanceTypes_returnsTotal() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(
                btcAsset,
                available = BigInteger("50000000"),
                staked = BigInteger("30000000"),
                pending = BigInteger("20000000"),
            ),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false)

        assertEquals("1 BTC", aggregate.balance)
    }

    @Test
    fun assetInfoDataAggregate_price_formatsValueChangeAndDirection() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = 2.5),
        )
        val price = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false).price

        assertEquals("\$50,000.00", price.price?.text())
        assertEquals("+2.50%", price.change?.text())
        assertEquals(GemValueTone.POSITIVE, price.change?.tone)
    }

    @Test
    fun assetInfoDataAggregate_price_nonFinitePrice_formatsEmptyValue() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = Double.NaN, priceChangePercentage24h = -5.2),
        )
        val price = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false).price

        assertNull("a price that is not a number is no price", price.price)
        assertEquals("-5.20%", price.change?.text())
        assertEquals(GemValueTone.NEGATIVE, price.change?.tone)
    }

    @Test
    fun toAssetInfoDataAggregates_matchesMappingEachItem() {
        val items = listOf(
            mockAssetInfo(
                asset = btcAsset,
                balance = AssetBalance.create(btcAsset, available = BigInteger("150000000")),
                price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = 1.0),
            ),
            mockAssetInfo(
                asset = ethAsset,
                balance = AssetBalance.create(ethAsset, available = BigInteger("2000000000000000000")),
                price = mockAssetPriceInfo(price = 3000.0, priceChangePercentage24h = -1.0, currency = Currency.EUR),
            ),
            mockAssetInfo(asset = btcAsset, price = null),
        )

        assertEquals(items.map { it.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = false) }, items.toAssetInfoDataAggregates(mockGemAssetRowStyle(), hideBalance = false))
        assertEquals(items.map { it.toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = true) }, items.toAssetInfoDataAggregates(mockGemAssetRowStyle(), hideBalance = true))
    }
}
