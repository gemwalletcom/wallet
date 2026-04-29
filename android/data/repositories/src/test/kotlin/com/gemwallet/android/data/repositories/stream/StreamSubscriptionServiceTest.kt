package com.gemwallet.android.data.repositories.stream

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.StreamMessage
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class StreamSubscriptionServiceTest {

    private val assetsDao = mockk<AssetsDao>()
    private val priceAlertsDao = mockk<PriceAlertsDao>()

    @Test
    fun `resubscribe emits full subscription to active collector`() = runTest {
        coEvery { assetsDao.getAssetsPriceUpdate("wallet-1") } returns listOf("bitcoin")
        every { priceAlertsDao.getAlerts() } returns flowOf(emptyList())
        val service = StreamSubscriptionService(
            assetsDao = assetsDao,
            priceAlertsDao = priceAlertsDao,
            scope = this,
        )

        service.setupAssets("wallet-1")
        val messages = mutableListOf<StreamMessage>()
        val collector = async { service.messageFlow.toList(messages) }

        service.resubscribe().join()
        collector.cancel()

        val subscribe = messages.last() as StreamMessage.SubscribePrices
        assertEquals(listOf(Chain.Bitcoin), subscribe.data.assets.map { it.chain })
    }
}
