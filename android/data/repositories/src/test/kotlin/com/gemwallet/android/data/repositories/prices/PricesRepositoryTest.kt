package com.gemwallet.android.data.repositories.prices

import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.DbFiatRate
import com.gemwallet.android.data.service.store.database.entities.DbPrice
import com.gemwallet.android.testkit.mockAssetBasic
import com.gemwallet.android.testkit.mockAssetFull
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockPrice
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatRate
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.slot
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class PricesRepositoryTest {

    private val pricesDao = mockk<PricesDao>(relaxed = true)
    private val sessionRepository = mockk<SessionRepository>(relaxed = true)

    private val subject = PricesRepository(
        pricesDao = pricesDao,
        sessionRepository = sessionRepository,
    )

    @Test
    fun updatePrice_assetWithoutPrice_storesZeroedRow() = runTest {
        val assetFull = mockAssetFull(asset = mockAssetSolana(), price = null)
        val stored = slot<DbPrice>()

        subject.updatePrice(assetFull, FiatRate(Currency.EUR.string, 0.5), Currency.EUR)

        coVerify(exactly = 1) { pricesDao.insert(capture(stored)) }
        assertEquals("solana", stored.captured.assetId)
        assertEquals("EUR", stored.captured.currency)
        assertEquals(0.0, stored.captured.value ?: -1.0, 0.0)
        assertEquals(0.0, stored.captured.usdValue ?: -1.0, 0.0)
    }

    @Test
    fun updatePrices_noAssetCarriesPrice_storesNothing() = runTest {
        every { pricesDao.getRates(Currency.USD) } returns flowOf(DbFiatRate(Currency.USD, 1.0))

        subject.updatePrices(listOf(mockAssetBasic(asset = mockAssetSolana())), Currency.USD)

        coVerify(exactly = 0) { pricesDao.insert(any<List<DbPrice>>()) }
    }

    @Test
    fun updatePrices_withoutStoredRate_storesNothing() = runTest {
        val asset = mockAssetBasic(asset = mockAssetSolana()).copy(price = mockPrice(price = 100.0))
        every { pricesDao.getRates(Currency.EUR) } returns flowOf(null)

        subject.updatePrices(listOf(asset), Currency.EUR)

        coVerify(exactly = 0) { pricesDao.insert(any<List<DbPrice>>()) }
    }
}
