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
import com.gemwallet.android.domains.pricealerts.aggregates.PriceAlertDataAggregate
import com.wallet.core.primitives.Price
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertData
import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemPriceAlertText
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
        priceAlert = priceAlert,
        row = PriceAlertFormatter().row(
            data = PriceAlertData(
                asset = asset,
                price = assetPrice?.price?.let {
                    Price(
                        price = it.price,
                        priceChangePercentage24h = it.priceChangePercentage24h,
                        updatedAt = it.updatedAt,
                    )
                },
                priceAlert = priceAlert,
            ).toGem(),
            priceCurrency = (assetPrice?.currency ?: priceAlert.currency).toGem(),
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

    private fun PriceAlertDataAggregate.number(currency: Boolean): GemFormattedNumber? =
        listOf(prefix, suffix)
            .mapNotNull { (it as? GemPriceAlertText.Number)?.value }
            .firstOrNull { (it.unit is GemNumberUnit.Currency) == currency }

    private fun PriceAlertDataAggregate.price(): GemFormattedNumber? = number(currency = true)

    private fun PriceAlertDataAggregate.percent(): GemFormattedNumber? = number(currency = false)

    @Test
    fun testPrice_fromPriceAlert() {
        val aggregate = createAggregate(priceAlert = mockPriceAlert(price = 50000.0, currency = Currency.USD))

        assertEquals(50000.0, aggregate.price()?.value)
        assertEquals(Currency.USD.string, (aggregate.price()?.unit as GemNumberUnit.Currency).code)
    }

    @Test
    fun testPrice_fromAssetPrice_whenAlertPriceNull() {
        val aggregate = createAggregate(
            assetPrice = mockAssetPriceInfo(price = 45234.50, currency = Currency.USD),
            priceAlert = mockPriceAlert(price = null),
        )

        assertEquals(45234.50, aggregate.price()?.value)
    }

    @Test
    fun testPrice_takesTheAlertCurrencyOverTheDisplayCurrency() {
        val aggregate = createAggregate(
            assetPrice = mockAssetPriceInfo(price = 42000.0, currency = Currency.EUR),
            priceAlert = mockPriceAlert(price = null, currency = Currency.EUR),
        )

        assertEquals(Currency.EUR.string, (aggregate.price()?.unit as GemNumberUnit.Currency).code)
    }

    @Test
    fun testPrice_withoutAnyPrice_hasNoNumber() {
        val aggregate = createAggregate(
            assetPrice = null,
            priceAlert = mockPriceAlert(price = null),
        )

        assertEquals(null, aggregate.price())
        assertEquals(GemPriceAlertText.Empty, aggregate.prefix)
    }

    @Test
    fun testPercentage_fromPriceAlert() {
        val aggregate = createAggregate(priceAlert = mockPriceAlert(pricePercentChange = 3.5))

        assertEquals(3.5, aggregate.percent()?.value)
    }

    @Test
    fun testPercentage_fromAssetPrice_whenAlertPercentNull() {
        val aggregate = createAggregate(
            assetPrice = mockAssetPriceInfo(priceChangePercentage24h = -2.15),
            priceAlert = mockPriceAlert(pricePercentChange = null),
        )

        assertEquals(-2.15, aggregate.percent()?.value)
    }

    @Test
    fun testPercentage_withoutAnyPercent_hasNoNumber() {
        val aggregate = createAggregate(
            assetPrice = null,
            priceAlert = mockPriceAlert(pricePercentChange = null),
        )

        assertEquals(null, aggregate.percent())
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
