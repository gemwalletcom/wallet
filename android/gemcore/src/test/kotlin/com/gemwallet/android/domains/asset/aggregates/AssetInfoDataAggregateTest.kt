package com.gemwallet.android.domains.asset.aggregates

import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetBalance
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetMetaData
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockGemAssetRowStyle
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemAssetBalanceScope
import uniffi.gemstone.GemAssetItemTrailing
import uniffi.gemstone.GemAssetSubtitleStyle
import uniffi.gemstone.GemAssetTitleStyle
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemRowText
import java.math.BigInteger

class AssetInfoDataAggregateTest {

    private val btcAsset = mockAsset()

    private val ethAsset = mockAssetEthereum()

    @Test
    fun theAggregateKeepsTheAssetIdentityNextToItsRow() {
        val account = mockAccount(address = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh")
        val aggregate = mockAssetInfo(asset = btcAsset, owner = account, metadata = mockAssetMetaData(isPinned = true)).toAssetInfoDataAggregate(mockGemAssetRowStyle(), hideBalance = true)

        assertEquals(btcAsset, aggregate.asset)
        assertEquals(btcAsset.id, aggregate.id)
        assertEquals(account.address, aggregate.accountAddress)
        assertTrue(aggregate.pinned)
        assertTrue(aggregate.hideBalance)
        assertTrue("a balance row is the one the privacy setting hides", aggregate.row.masksBalance)
        assertEquals("", mockAssetInfo(asset = btcAsset, owner = null).toAssetInfoDataAggregate(mockGemAssetRowStyle()).accountAddress)
    }

    @Test
    fun theTitleSymbolAndNetworkAreTheOnesCoreResolved() {
        val usdc = mockAsset(chain = Chain.Ethereum, tokenId = "0xusdc", name = "USDC", symbol = "USDC")
        val assetInfo = mockAssetInfo(asset = usdc)
        val networked = assetInfo.toAssetInfoDataAggregate(
            mockGemAssetRowStyle(title = GemAssetTitleStyle.NETWORK, showsSymbol = true, subtitle = GemAssetSubtitleStyle.NETWORK),
        ).row

        assertEquals("Ethereum", networked.title)
        assertEquals("USDC", networked.titleExtra)
        assertEquals("Ethereum", networked.subtitle.string)

        val named = assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(showsSymbol = true)).row

        assertEquals("USDC", named.title)
        assertNull("a symbol that repeats the title is not shown", named.titleExtra)
    }

    @Test
    fun theBalanceTrailsWithItsValueInThePriceCurrency() {
        val aggregate = mockAssetInfo(
            asset = btcAsset,
            balance = mockAssetBalance(btcAsset, available = BigInteger("100000000")),
            price = mockAssetPriceInfo(price = 3000.0, priceChangePercentage24h = -5.0, currency = Currency.EUR),
        ).toAssetInfoDataAggregate(mockGemAssetRowStyle())
        val trailing = aggregate.row.trailing as GemAssetItemTrailing.Value

        assertEquals("1 BTC", trailing.value.string)
        assertEquals("€3,000.00", trailing.extra.string)
        assertEquals("€3,000.00", aggregate.row.subtitle.string)
        assertEquals("-5.00%", aggregate.row.subtitleExtra.string)
    }

    @Test
    fun theAvailableScopeShowsWhatIsSpendable() {
        val assetInfo = mockAssetInfo(
            asset = btcAsset,
            balance = mockAssetBalance(btcAsset, available = BigInteger("100000000"), staked = BigInteger("200000000")),
        )

        assertEquals("3 BTC", assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle()).row.trailingValue.string)
        assertEquals("1 BTC", assetInfo.toAssetInfoDataAggregate(mockGemAssetRowStyle(), scope = GemAssetBalanceScope.AVAILABLE).row.trailingValue.string)
    }

    @Test
    fun aPriceThatIsNotANumberIsNoPrice() {
        val row = mockAssetInfo(
            asset = btcAsset,
            balance = mockAssetBalance(btcAsset, available = BigInteger("100000000")),
            price = mockAssetPriceInfo(price = Double.NaN, priceChangePercentage24h = -5.2),
        ).toAssetInfoDataAggregate(mockGemAssetRowStyle()).row

        assertNull(row.subtitle)
        assertNull((row.trailing as GemAssetItemTrailing.Value).extra)
    }

    @Test
    fun theListMatchesMappingEachItemAndWalletListsUseTheWalletStyle() {
        val items = listOf(
            mockAssetInfo(
                asset = btcAsset,
                balance = mockAssetBalance(btcAsset, available = BigInteger("150000000")),
                price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = 1.0),
            ),
            mockAssetInfo(
                asset = ethAsset,
                balance = mockAssetBalance(ethAsset, available = BigInteger("2000000000000000000")),
                price = mockAssetPriceInfo(price = 3000.0, priceChangePercentage24h = -1.0, currency = Currency.EUR),
            ),
            mockAssetInfo(asset = btcAsset, price = null),
        )
        val style = mockGemAssetRowStyle(title = GemAssetTitleStyle.CANONICAL_ASSET)

        assertEquals(items.map { it.toAssetInfoDataAggregate(style, hideBalance = true) }, items.toAssetInfoDataAggregates(style, hideBalance = true))
        assertEquals(items.toAssetInfoDataAggregates(style), items.toAssetInfoDataAggregates())
    }
}

private val GemRowText?.string: String?
    get() = when (val text = this?.text) {
        is GemLocalizedText.Number -> text.number.text()
        is GemLocalizedText.Text -> text.text
        else -> null
    }
