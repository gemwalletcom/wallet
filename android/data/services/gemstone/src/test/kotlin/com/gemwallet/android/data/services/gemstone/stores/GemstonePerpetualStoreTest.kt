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
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

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
    fun `a query without stored priorities filters by name and symbol`() = runTest {
        every { searchDao.hasPerpetualPriorities(any()) } returns flowOf(0)
        every { perpetualDao.getPerpetualsData() } returns flowOf(listOf(bitcoin, ethereum))

        assertEquals(listOf("BTC-PERP"), store.observePerpetuals("bitcoin").first().map { it.perpetual.identifier })
        assertEquals(listOf("ETH-PERP"), store.observePerpetuals("eth").first().map { it.perpetual.identifier })
        assertEquals(emptyList<String>(), store.observePerpetuals("doge").first())
        assertEquals(2, store.observePerpetuals(null).first().size)
    }

    @Test
    fun `a query with stored priorities uses the priority order from the database`() = runTest {
        every { searchDao.hasPerpetualPriorities("btc") } returns flowOf(1)
        every { perpetualDao.searchWithPriority("btc") } returns flowOf(listOf(ethereum, bitcoin))

        val result = store.observePerpetuals("btc").first()

        assertEquals(listOf("ETH-PERP", "BTC-PERP"), result.map { it.perpetual.identifier })
    }
}
