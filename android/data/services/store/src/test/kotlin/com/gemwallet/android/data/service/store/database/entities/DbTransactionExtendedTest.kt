package com.gemwallet.android.data.service.store.database.entities

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionType
import org.junit.Assert.assertEquals
import org.junit.Test

class DbTransactionExtendedTest {

    private val eth = Asset(AssetId(Chain.Ethereum), "Ethereum", "ETH", 18, AssetType.NATIVE)
    private val usdt = Asset(AssetId(Chain.Ethereum, "0xdac17f958d2ee523a2206206994597c13d831ec7"), "Tether", "USDT", 6, AssetType.ERC20)

    @Test
    fun toDTO_pricesBothSwapLegs() {
        val extended = mockDbTransactionExtended(
            type = TransactionType.Swap,
            fromAsset = eth,
            toAsset = usdt,
            fromPriceValue = 3000.0,
            fromPriceDayChanged = 1.5,
            toPriceValue = 1.0,
        ).toDTO()

        assertEquals(listOf(eth, usdt), extended?.assets)
        assertEquals(
            listOf(AssetPrice(eth.id, 3000.0, 1.5, 0L), AssetPrice(usdt.id, 1.0, 0.0, 0L)),
            extended?.prices,
        )
    }

    @Test
    fun toDTO_skipsSwapLegsWithoutAPrice() {
        val extended = mockDbTransactionExtended(type = TransactionType.Swap, fromAsset = eth, toAsset = usdt, toPriceValue = 1.0).toDTO()

        assertEquals(listOf(usdt.id), extended?.prices?.map { it.assetId })
    }

    @Test
    fun toDTO_hasNoPricesWithoutSwapLegs() {
        val extended = mockDbTransactionExtended(priceValue = 3000.0).toDTO()

        assertEquals(3000.0, extended?.price?.price)
        assertEquals(emptyList<AssetPrice>(), extended?.prices)
    }
}
