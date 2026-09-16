package com.gemwallet.android.data.service.store.database.entities

import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetEthereumUSDT
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.TransactionType
import org.junit.Assert.assertEquals
import org.junit.Test

class DbTransactionExtendedTest {

    private val eth = mockAssetEthereum()
    private val usdt = mockAssetEthereumUSDT()

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
