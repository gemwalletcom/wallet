package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PerpetualDao
import com.gemwallet.android.data.service.store.database.PerpetualPositionDao
import com.gemwallet.android.data.service.store.database.SearchDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.entities.mockDbPerpetualData
import com.gemwallet.android.testkit.mockAssetEthereum
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.perpetualMarketQuery

class GemstonePerpetualStoreTest {

    private val perpetualDao = mockk<PerpetualDao>()
    private val searchDao = mockk<SearchDao>()

    private val bitcoin = mockDbPerpetualData()
    private val ethereum = mockDbPerpetualData(asset = mockAssetEthereum(), identifier = "ETH-PERP")

    private val store = GemstonePerpetualStore(
        perpetualDao = perpetualDao,
        searchDao = searchDao,
        perpetualPositionDao = mockk<PerpetualPositionDao>(),
        balancesDao = mockk<BalancesDao>(),
        transactionRunner = mockk<StoreTransactionRunner>(),
    )

    @Test
    fun `a query without stored priorities matches name and symbol in the database`() = runTest {
        every { searchDao.hasPerpetualPriorities(any()) } returns flowOf(0)
        every { perpetualDao.searchPerpetualsData("bitcoin", MARKETS_LIMIT) } returns flowOf(listOf(bitcoin))
        every { perpetualDao.searchPerpetualsData("doge", MARKETS_LIMIT) } returns flowOf(emptyList())

        assertEquals(listOf("BTC-PERP"), store.observePerpetuals(perpetualMarketQuery("bitcoin")).first().map { it.perpetual.identifier })
        assertEquals(emptyList<String>(), store.observePerpetuals(perpetualMarketQuery("doge")).first())
    }

    @Test
    fun `a query with stored priorities uses the priority order from the database`() = runTest {
        every { searchDao.hasPerpetualPriorities("btc") } returns flowOf(1)
        every { perpetualDao.searchWithPriority("btc", MARKETS_LIMIT) } returns flowOf(listOf(ethereum, bitcoin))

        val result = store.observePerpetuals(perpetualMarketQuery("btc")).first()

        assertEquals(listOf("ETH-PERP", "BTC-PERP"), result.map { it.perpetual.identifier })
    }

    @Test
    fun `browsing asks for traded markets and a search asks for every one`() = runTest {
        every { searchDao.hasPerpetualPriorities(any()) } returns flowOf(0)
        every { perpetualDao.getPerpetualsData(true, MARKETS_LIMIT) } returns flowOf(listOf(bitcoin, ethereum))
        every { perpetualDao.searchPerpetualsData("btc", MARKETS_LIMIT) } returns flowOf(listOf(bitcoin))

        assertEquals(2, store.observePerpetuals(perpetualMarketQuery("")).first().size)
        assertEquals(1, store.observePerpetuals(perpetualMarketQuery("  btc ")).first().size)

        verify { perpetualDao.getPerpetualsData(true, MARKETS_LIMIT) }
        verify(exactly = 0) { perpetualDao.getPerpetualsData(false, any()) }
    }

    private companion object {
        const val MARKETS_LIMIT = 100
    }
}
