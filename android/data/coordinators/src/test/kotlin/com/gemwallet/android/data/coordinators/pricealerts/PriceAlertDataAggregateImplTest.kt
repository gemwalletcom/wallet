package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AssetPriceInfo
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockPriceAlert
import uniffi.gemstone.PriceAlertFormatter
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlert
import org.junit.Assert.assertEquals
import org.junit.Test

class PriceAlertDataAggregateImplTest {

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

    private val solAsset = Asset(
        id = AssetId(Chain.Solana),
        name = "Solana",
        symbol = "sol",
        decimals = 9,
        type = AssetType.NATIVE,
    )

    private fun createAggregate(
        id: String = "1",
        asset: Asset = btcAsset,
        assetPrice: AssetPriceInfo? = mockAssetPriceInfo(),
        priceAlert: PriceAlert = mockPriceAlert(assetId = asset.id),
    ) = PriceAlertDataAggregateImpl(
        id = id,
        asset = asset,
        assetPrice = assetPrice,
        priceAlert = priceAlert,
        row = PriceAlertFormatter().row(
            alert = priceAlert.toGem(),
            currentPrice = assetPrice?.price?.price,
            priceChangePercentage24h = assetPrice?.price?.priceChangePercentage24h,
            priceCurrency = (assetPrice?.currency ?: priceAlert.currency).string,
        ),
    )

    @Test
    fun testBasicPropertyDelegation() {
        val aggregate = createAggregate(
            id = "123",
            asset = ethAsset,
        )

        assertEquals("123", aggregate.id)
        assertEquals(ethAsset, aggregate.asset)
        assertEquals(ethAsset.id, aggregate.assetId)
        assertEquals("Ethereum", aggregate.title)
    }

    @Test
    fun testTitleBadge_uppercase() {
        val aggregate = createAggregate(asset = solAsset)

        assertEquals("SOL", aggregate.titleBadge)
    }

    @Test
    fun testPrice_fromPriceAlert() {
        val priceAlert = mockPriceAlert(
            price = 50000.0,
            currency = Currency.USD,
        )
        val aggregate = createAggregate(priceAlert = priceAlert)

        assertEquals("$50,000.00", aggregate.price)
    }

    @Test
    fun testPrice_fromAssetPrice_whenAlertPriceNull() {
        val assetPrice = mockAssetPriceInfo(
            price = 45234.50,
            currency = Currency.USD,
        )
        val priceAlert = mockPriceAlert(price = null)
        val aggregate = createAggregate(
            assetPrice = assetPrice,
            priceAlert = priceAlert,
        )

        assertEquals("$45,234.50", aggregate.price)
    }

    @Test
    fun testPrice_fromAssetPrice_whenSmallValue_usesDynamicFormatting() {
        val assetPrice = mockAssetPriceInfo(
            price = 0.0000111,
            currency = Currency.USD,
        )
        val priceAlert = mockPriceAlert(price = null)
        val aggregate = createAggregate(
            assetPrice = assetPrice,
            priceAlert = priceAlert,
        )

        assertEquals("$0.0000111", aggregate.price)
    }

    @Test
    fun testPrice_withEuroCurrency() {
        val assetPrice = mockAssetPriceInfo(
            price = 42000.0,
            currency = Currency.EUR,
        )
        val priceAlert = mockPriceAlert(
            price = null,
            currency = Currency.EUR,
        )
        val aggregate = createAggregate(
            assetPrice = assetPrice,
            priceAlert = priceAlert,
        )

        assertEquals("€42,000.00", aggregate.price)
    }

    @Test
    fun testPrice_fromPriceAlert_whenSmallValue_usesDynamicFormatting() {
        val priceAlert = mockPriceAlert(
            price = 0.006333,
            currency = Currency.USD,
        )
        val aggregate = createAggregate(priceAlert = priceAlert)

        assertEquals("$0.006333", aggregate.price)
    }

    @Test
    fun testPrice_withoutAssetPrice_returnsEmpty() {
        val aggregate = createAggregate(
            assetPrice = null,
            priceAlert = mockPriceAlert(price = null),
        )

        assertEquals("", aggregate.price)
    }

    @Test
    fun testPercentage_fromPriceAlert() {
        val priceAlert = mockPriceAlert(
            pricePercentChange = 3.5,
        )
        val aggregate = createAggregate(priceAlert = priceAlert)

        assertEquals("3.50%", aggregate.percentage)
    }

    @Test
    fun testPercentage_fromAssetPrice_whenAlertPercentNull() {
        val assetPrice = mockAssetPriceInfo(
            priceChangePercentage24h = -2.15,
        )
        val priceAlert = mockPriceAlert(
            pricePercentChange = null,
        )
        val aggregate = createAggregate(
            assetPrice = assetPrice,
            priceAlert = priceAlert,
        )

        assertEquals("-2.15%", aggregate.percentage)
    }

    @Test
    fun testPercentage_largeValue() {
        val priceAlert = mockPriceAlert(
            pricePercentChange = 125.67,
        )
        val aggregate = createAggregate(priceAlert = priceAlert)

        assertEquals("125.67%", aggregate.percentage)
    }

    @Test
    fun testPercentage_withoutAssetPrice_returnsEmpty() {
        val aggregate = createAggregate(
            assetPrice = null,
            priceAlert = mockPriceAlert(pricePercentChange = null),
        )

        assertEquals("", aggregate.percentage)
    }

    @Test
    fun testMultipleAssets() {
        val btcAggregate = createAggregate(
            id = "1",
            asset = btcAsset,
            assetPrice = mockAssetPriceInfo(price = 45000.0),
        )
        val ethAggregate = createAggregate(
            id = "2",
            asset = ethAsset,
            assetPrice = mockAssetPriceInfo(price = 2500.0),
        )
        val solAggregate = createAggregate(
            id = "3",
            asset = solAsset,
            assetPrice = mockAssetPriceInfo(price = 98.5),
        )

        assertEquals("1", btcAggregate.id)
        assertEquals("2", ethAggregate.id)
        assertEquals("3", solAggregate.id)
        assertEquals("Bitcoin", btcAggregate.title)
        assertEquals("Ethereum", ethAggregate.title)
        assertEquals("Solana", solAggregate.title)
    }
}
