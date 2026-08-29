package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PerpetualDao
import com.gemwallet.android.data.service.store.database.PerpetualPositionDao
import com.gemwallet.android.data.service.store.database.SearchDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.entities.DbAsset
import com.gemwallet.android.data.service.store.database.entities.DbPerpetual
import com.gemwallet.android.data.service.store.database.entities.DbPerpetualData
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class GemstonePerpetualStoreTest {

    private val perpetualDao = mockk<PerpetualDao>()
    private val searchDao = mockk<SearchDao>()

    private val store = GemstonePerpetualStore(
        perpetualDao = perpetualDao,
        searchDao = searchDao,
        perpetualPositionDao = mockk<PerpetualPositionDao>(),
        balancesDao = mockk<BalancesDao>(),
        transactionRunner = mockk<StoreTransactionRunner>(),
    )

    @Test
    fun `a query without stored priorities filters by name and symbol`() = runTest {
        every { searchDao.hasPerpetualPriorities(any()) } returns flowOf(0)
        every { perpetualDao.getPerpetualsData() } returns flowOf(listOf(bitcoin(), ethereum()))

        assertEquals(listOf("BTC-PERP"), store.observePerpetuals("bitcoin").first().map { it.perpetual.identifier })
        assertEquals(listOf("ETH-PERP"), store.observePerpetuals("eth").first().map { it.perpetual.identifier })
        assertEquals(emptyList<String>(), store.observePerpetuals("doge").first())
        assertEquals(2, store.observePerpetuals(null).first().size)
    }

    @Test
    fun `a query with stored priorities uses the priority order from the database`() = runTest {
        every { searchDao.hasPerpetualPriorities("btc") } returns flowOf(1)
        every { perpetualDao.searchWithPriority("btc") } returns flowOf(listOf(ethereum(), bitcoin()))

        val result = store.observePerpetuals("btc").first()

        assertEquals(listOf("ETH-PERP", "BTC-PERP"), result.map { it.perpetual.identifier })
    }

    private fun bitcoin() = perpetualData(identifier = "BTC-PERP", name = "Bitcoin Perpetual", asset = mockAsset())

    private fun ethereum() = perpetualData(identifier = "ETH-PERP", name = "Ethereum Perpetual", asset = mockAssetEthereum())

    private fun perpetualData(identifier: String, name: String, asset: Asset) = DbPerpetualData(
        perpetual = DbPerpetual(
            id = PerpetualId(PerpetualProvider.Hypercore, identifier),
            name = name,
            provider = PerpetualProvider.Hypercore,
            assetId = asset.id,
            identifier = identifier,
            price = 1.0,
            pricePercentChange24h = 0.0,
            openInterest = 0.0,
            volume24h = 0.0,
            funding = 0.0,
            maxLeverage = 1,
        ),
        asset = DbAsset(
            id = asset.id.toIdentifier(),
            name = asset.name,
            symbol = asset.symbol,
            decimals = asset.decimals,
            type = asset.type,
            chain = asset.id.chain,
        ),
    )
}
