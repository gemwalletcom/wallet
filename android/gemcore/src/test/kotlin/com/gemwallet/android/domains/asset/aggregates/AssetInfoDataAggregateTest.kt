package com.gemwallet.android.domains.asset.aggregates

import com.gemwallet.android.testkit.mockAssetMetaData
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.AssetPriceInfo
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetMetaData
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class AssetInfoDataAggregateTest {

    private val btcAsset = Asset(
        id = AssetId(Chain.Bitcoin),
        name = "Bitcoin",
        symbol = "BTC",
        decimals = 8,
        type = AssetType.NATIVE,
    )

    private val ethAsset = Asset(
        id = AssetId(Chain.Ethereum),
        name = "Ethereum",
        symbol = "ETH",
        decimals = 18,
        type = AssetType.NATIVE,
    )

    private val account = Account(
        chain = Chain.Bitcoin,
        address = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh",
        derivationPath = "m/84'/0'/0'/0/0"
    )

    @Test
    fun assetInfoDataAggregate_id_returnsAssetId() {
        val assetInfo = createAssetInfo(asset = btcAsset)
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals(btcAsset.id, aggregate.id)
    }

    @Test
    fun assetInfoDataAggregate_title_returnsAssetName() {
        val assetInfo = createAssetInfo(asset = btcAsset)
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals("Bitcoin", aggregate.title)
    }

    @Test
    fun assetInfoDataAggregate_asset_returnsAsset() {
        val assetInfo = createAssetInfo(asset = btcAsset)
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals(btcAsset, aggregate.asset)
    }

    @Test
    fun assetInfoDataAggregate_balance_hideBalanceTrue_returnsStars() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = "100000000")
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = true)

        assertEquals("*****", aggregate.balance)
    }

    @Test
    fun assetInfoDataAggregate_balance_hideBalanceFalse_returnsFormattedBalance() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = "100000000")
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)
        assertEquals("1 BTC", aggregate.balance)
    }

    @Test
    fun assetInfoDataAggregate_balance_zeroBalance_returnsFormattedZero() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = "0")
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals("0 BTC", aggregate.balance)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_hideBalanceTrue_returnsStars() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = "100000000"),
            price = createAssetPriceInfo(price = 50000.0)
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = true)

        assertEquals("*****", aggregate.balanceEquivalent)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_withPrice_returnsFormattedFiat() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = "100000000"),
            price = createAssetPriceInfo(price = 50000.0)
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)
        assertEquals("\$50,000.00", aggregate.balanceEquivalent)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_noPrice_returnsEmpty() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = "100000000"),
            price = null
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals("", aggregate.balanceEquivalent)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_zeroPriceValue_returnsEmpty() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = "100000000"),
            price = createAssetPriceInfo(price = 0.0)
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals("", aggregate.balanceEquivalent)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_nonFinitePrice_returnsEmpty() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = "100000000"),
            price = createAssetPriceInfo(price = Double.NaN)
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals("", aggregate.balanceEquivalent)
        assertEquals("", aggregate.price?.valueFormatted)
    }

    @Test
    fun assetInfoDataAggregate_isZeroBalance_zeroBalance_returnsTrue() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(btcAsset, available = "0")
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertTrue(aggregate.isZeroBalance)
    }

    @Test
    fun assetInfoDataAggregate_price_withPrice_returnsPriceableValue() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            price = createAssetPriceInfo(price = 50000.0, priceChangePercentage24h = -5.000002)
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertNotNull(aggregate.price)
        assertEquals(Currency.USD, aggregate.price?.currency)
        assertEquals(50000.0, aggregate.price?.value)
        assertEquals("\$50,000.00", aggregate.price?.valueFormatted)
        assertEquals(-5.000002, aggregate.price?.changePercentage)
        assertEquals("-5.00%", aggregate.price?.changePercentageFormatted)
        assertEquals(ValueDirection.Down, aggregate.price?.state)
    }

    @Test
    fun assetInfoDataAggregate_small_negative_change_value() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            price = createAssetPriceInfo(price = 50000.0, priceChangePercentage24h = -0.000006)
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals(-0.000006, aggregate.price?.changePercentage)
        assertEquals("-0.00%", aggregate.price?.changePercentageFormatted)
        assertEquals(ValueDirection.Down, aggregate.price?.state)
    }

    @Test
    fun assetInfoDataAggregate_small_positive_change_value() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            price = createAssetPriceInfo(price = 50000.0, priceChangePercentage24h = 0.000006)
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals(0.000006, aggregate.price?.changePercentage)
        assertEquals("+0.00%", aggregate.price?.changePercentageFormatted)
        assertEquals(ValueDirection.Up, aggregate.price?.state)
    }


    @Test
    fun assetInfoDataAggregate_small_change_value() {
        createAssetInfo(
            asset = btcAsset,
            price = createAssetPriceInfo(price = 50000.0, priceChangePercentage24h = -0.06)
        ).toAssetInfoDataAggregate(hideBalance = false).let { aggregate ->
            assertEquals("-0.06%", aggregate.price?.changePercentageFormatted)
            assertEquals(ValueDirection.Down, aggregate.price?.state)
        }
        createAssetInfo(
            asset = btcAsset,
            price = createAssetPriceInfo(price = 50000.0, priceChangePercentage24h = -0.02)
        ).toAssetInfoDataAggregate(hideBalance = false).let { aggregate ->
            assertEquals("-0.02%", aggregate.price?.changePercentageFormatted)
            assertEquals(ValueDirection.Down, aggregate.price?.state)
        }

        createAssetInfo(
            asset = btcAsset,
            price = createAssetPriceInfo(price = 50000.0, priceChangePercentage24h = 0.06)
        ).toAssetInfoDataAggregate(hideBalance = false).let { aggregate ->
            assertEquals("+0.06%", aggregate.price?.changePercentageFormatted)
            assertEquals(ValueDirection.Up, aggregate.price?.state)
        }

        createAssetInfo(
            asset = btcAsset,
            price = createAssetPriceInfo(price = 50000.0, priceChangePercentage24h = 0.02)
        ).toAssetInfoDataAggregate(hideBalance = false).let { aggregate ->
            assertEquals("+0.02%", aggregate.price?.changePercentageFormatted)
            assertEquals(ValueDirection.Up, aggregate.price?.state)
        }
    }

    @Test
    fun assetInfoDataAggregate_price_noPrice_returnsNull() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            price = null
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertNull(aggregate.price)
    }

    @Test
    fun assetInfoDataAggregate_pinned_pinnedTrue_returnsTrue() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            metadata = AssetMetaData(
                isEnabled = true,
                isBalanceEnabled = true,
                isBuyEnabled = true,
                isSellEnabled = true,
                isSwapEnabled = true,
                isStakeEnabled = false,
                isEarnEnabled = false,
                isPinned = true,
                isActive = true,
                stakingApr = null,
                earnApr = null,
                rankScore = 0
            )
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertTrue(aggregate.pinned)
    }

    @Test
    fun assetInfoDataAggregate_pinned_pinnedFalse_returnsFalse() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            metadata = AssetMetaData(
                isEnabled = true,
                isBalanceEnabled = true,
                isBuyEnabled = true,
                isSellEnabled = true,
                isSwapEnabled = true,
                isStakeEnabled = false,
                isEarnEnabled = false,
                isPinned = false,
                isActive = true,
                stakingApr = null,
                earnApr = null,
                rankScore = 0
            )
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertFalse(aggregate.pinned)
    }

    @Test
    fun assetInfoDataAggregate_accountAddress_withOwner_returnsAddress() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            owner = account
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals("bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh", aggregate.accountAddress)
    }

    @Test
    fun assetInfoDataAggregate_accountAddress_noOwner_returnsEmpty() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            owner = null
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals("", aggregate.accountAddress)
    }

    @Test
    fun assetInfoDataAggregate_price_withEuroCurrency_returnsEuro() {
        val assetInfo = createAssetInfo(
            asset = ethAsset,
            price = createAssetPriceInfo(price = 3000.0, currency = Currency.EUR)
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertNotNull(aggregate.price)
        assertEquals(Currency.EUR, aggregate.price?.currency)
        assertEquals(3000.0, aggregate.price?.value)
    }

    @Test
    fun assetInfoDataAggregate_balanceEquivalent_withEuroCurrency_returnsEuroFormat() {
        val assetInfo = createAssetInfo(
            asset = ethAsset,
            balance = AssetBalance.create(ethAsset, available = "1000000000000000000"),
            price = createAssetPriceInfo(price = 3000.0, currency = Currency.EUR)
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals("€3,000.00", aggregate.balanceEquivalent)
    }

    @Test
    fun assetInfoDataAggregate_balance_multipleBalanceTypes_returnsTotal() {
        val assetInfo = createAssetInfo(
            asset = btcAsset,
            balance = AssetBalance.create(
                btcAsset,
                available = "50000000",
                staked = "30000000",
                pending = "20000000"
            )
        )
        val aggregate = assetInfo.toAssetInfoDataAggregate(hideBalance = false)

        assertEquals("1 BTC", aggregate.balance)
    }

    private fun createAssetInfo(
        asset: Asset,
        owner: Account? = null,
        balance: AssetBalance = AssetBalance.create(asset),
        walletId: String? = "wallet1",
        price: AssetPriceInfo? = null,
        metadata: AssetMetaData = mockAssetMetaData(),
    ): AssetInfo {
        return AssetInfo(
            owner = owner,
            asset = asset,
            balance = balance,
            walletId = walletId?.let(::mockWalletId),
            price = price,
            metadata = metadata,
        )
    }

    private fun createAssetPriceInfo(
        price: Double,
        priceChangePercentage24h: Double = 0.0,
        currency: Currency = Currency.USD
    ): AssetPriceInfo {
        return AssetPriceInfo(
            currency = currency,
            price = AssetPrice(
                assetId = btcAsset.id,
                price = price,
                priceChangePercentage24h = priceChangePercentage24h,
                updatedAt = 0L
            )
        )
    }
}
