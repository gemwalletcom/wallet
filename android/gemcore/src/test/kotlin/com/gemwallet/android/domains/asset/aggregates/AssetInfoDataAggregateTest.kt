package com.gemwallet.android.domains.asset.aggregates

import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetMetaData
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemValueTone
import java.math.BigInteger

class AssetInfoDataAggregateTest {

    private val btcAsset = mockAsset()

    private val ethAsset = mockAssetEthereum()

    private val account = mockAccount(address = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh")

    @Test
    fun assetInfoDataAggregate_id_returnsAssetId() {
        val assetInfo = mockAssetInfo(asset = btcAsset)
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals(btcAsset.id, aggregate.id)
    }

    @Test
    fun assetInfoDataAggregate_title_returnsAssetName() {
        val assetInfo = mockAssetInfo(asset = btcAsset)
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals("Bitcoin", aggregate.title)
    }

    @Test
    fun assetInfoDataAggregate_asset_returnsAsset() {
        val assetInfo = mockAssetInfo(asset = btcAsset)
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals(btcAsset, aggregate.asset)
    }

    @Test
    fun assetInfoDataAggregate_balance_hideBalanceTrue_returnsStars() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("100000000")),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = true)

        assertEquals("*****", aggregate.balance)
    }

    @Test
    fun assetInfoDataAggregate_balance_hideBalanceFalse_returnsFormattedBalance() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("100000000")),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)
        assertEquals("1 BTC", aggregate.balance)
    }

    @Test
    fun assetInfoDataAggregate_balance_zeroBalance_returnsFormattedZero() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("0")),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals("0 BTC", aggregate.balance)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_hideBalanceTrue_returnsStars() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("100000000")),
            price = mockAssetPriceInfo(price = 50000.0),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = true)

        assertEquals("*****", aggregate.balanceEquivalent)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_withPrice_returnsFormattedFiat() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("100000000")),
            price = mockAssetPriceInfo(price = 50000.0),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)
        assertEquals("\$50,000.00", aggregate.balanceEquivalent)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_noPrice_returnsEmpty() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("100000000")),
            price = null,
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals("", aggregate.balanceEquivalent)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_zeroPriceValue_returnsEmpty() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("100000000")),
            price = mockAssetPriceInfo(price = 0.0),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals("", aggregate.balanceEquivalent)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_nonFinitePrice_returnsEmpty() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("100000000")),
            price = mockAssetPriceInfo(price = Double.NaN),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals("", aggregate.balanceEquivalent)
        assertEquals("", aggregate.price?.valueFormatted)
    }

    @Test
    fun assetInfoDataAggregate_isZeroBalance_zeroBalance_returnsTrue() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = BigInteger("0")),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertTrue(aggregate.isZeroBalance)
    }

    @Test
    fun assetInfoDataAggregate_price_withPrice_returnsPriceableValue() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = -5.000002),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertNotNull(aggregate.price)
        assertEquals(Currency.USD, aggregate.price?.currency)
        assertEquals(50000.0, aggregate.price?.value)
        assertEquals("\$50,000.00", aggregate.price?.valueFormatted)
        assertEquals(-5.000002, aggregate.price?.changePercentage)
        assertEquals("-5.00%", aggregate.price?.changePercentageFormatted)
        assertEquals(GemValueTone.NEGATIVE, aggregate.price?.state)
    }

    @Test
    fun assetInfoDataAggregate_small_negative_change_value() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = -0.000006),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals(-0.000006, aggregate.price?.changePercentage)
        assertEquals("-0.00%", aggregate.price?.changePercentageFormatted)
        assertEquals(GemValueTone.NEGATIVE, aggregate.price?.state)
    }

    @Test
    fun assetInfoDataAggregate_small_positive_change_value() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = 0.000006),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals(0.000006, aggregate.price?.changePercentage)
        assertEquals("+0.00%", aggregate.price?.changePercentageFormatted)
        assertEquals(GemValueTone.POSITIVE, aggregate.price?.state)
    }

    @Test
    fun assetInfoDataAggregate_small_change_value() {
        mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = -0.06),
        ).toAssetInfoDataAggregate(hideBalance = false).let { aggregate ->
            assertEquals("-0.06%", aggregate.price?.changePercentageFormatted)
            assertEquals(GemValueTone.NEGATIVE, aggregate.price?.state)
        }
        mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = -0.02),
        ).toAssetInfoDataAggregate(hideBalance = false).let { aggregate ->
            assertEquals("-0.02%", aggregate.price?.changePercentageFormatted)
            assertEquals(GemValueTone.NEGATIVE, aggregate.price?.state)
        }

        mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = 0.06),
        ).toAssetInfoDataAggregate(hideBalance = false).let { aggregate ->
            assertEquals("+0.06%", aggregate.price?.changePercentageFormatted)
            assertEquals(GemValueTone.POSITIVE, aggregate.price?.state)
        }

        mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = 0.02),
        ).toAssetInfoDataAggregate(hideBalance = false).let { aggregate ->
            assertEquals("+0.02%", aggregate.price?.changePercentageFormatted)
            assertEquals(GemValueTone.POSITIVE, aggregate.price?.state)
        }
    }

    @Test
    fun assetInfoDataAggregate_price_noPrice_returnsNull() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            price = null,
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertNull(aggregate.price)
    }

    @Test
    fun assetInfoDataAggregate_pinned_pinnedTrue_returnsTrue() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            metadata = mockAssetMetaData(isPinned = true),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertTrue(aggregate.pinned)
    }

    @Test
    fun assetInfoDataAggregate_pinned_pinnedFalse_returnsFalse() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            metadata = mockAssetMetaData(isPinned = false),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertFalse(aggregate.pinned)
    }

    @Test
    fun assetInfoDataAggregate_accountAddress_withOwner_returnsAddress() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            owner = account,
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals("bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh", aggregate.accountAddress)
    }

    @Test
    fun assetInfoDataAggregate_accountAddress_noOwner_returnsEmpty() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            owner = null,
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals("", aggregate.accountAddress)
    }

    @Test
    fun assetInfoDataAggregate_price_withEuroCurrency_returnsEuro() {
        val assetInfo = mockAssetInfo(
            asset = ethAsset,
            price = mockAssetPriceInfo(price = 3000.0, currency = Currency.EUR),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertNotNull(aggregate.price)
        assertEquals(Currency.EUR, aggregate.price?.currency)
        assertEquals(3000.0, aggregate.price?.value)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_withEuroCurrency_returnsEuroFormat() {
        val assetInfo = mockAssetInfo(
            asset = ethAsset,
            balance = AssetBalance.create(ethAsset, available = BigInteger("1000000000000000000")),
            price = mockAssetPriceInfo(price = 3000.0, currency = Currency.EUR),
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

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
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals("1 BTC", aggregate.balance)
    }

    @Test
    fun assetInfoDataAggregate_price_formatsValueChangeAndDirection() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = 2.5),
        )
        val price = assetInfo.toAssetInfoDataAggregate(hideBalance = false).price

        assertEquals("\$50,000.00", price?.valueFormatted)
        assertEquals("+2.50%", price?.changePercentageFormatted)
        assertEquals(GemValueTone.POSITIVE, price?.state)
    }

    @Test
    fun assetInfoDataAggregate_price_nonFinitePrice_formatsEmptyValue() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            price = mockAssetPriceInfo(price = Double.NaN, priceChangePercentage24h = -5.2),
        )
        val price = assetInfo.toAssetInfoDataAggregate(hideBalance = false).price

        assertEquals("", price?.valueFormatted)
        assertEquals("-5.20%", price?.changePercentageFormatted)
        assertEquals(GemValueTone.NEGATIVE, price?.state)
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

        assertEquals(items.map { it.toAssetInfoDataAggregate(hideBalance = false) }, items.toAssetInfoDataAggregates(hideBalance = false))
        assertEquals(items.map { it.toAssetInfoDataAggregate(hideBalance = true) }, items.toAssetInfoDataAggregates(hideBalance = true))
    }
}
