package com.gemwallet.android.data.repositories.stream

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.StreamMessage
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class StreamSubscriptionServiceTest {

    private val assetsDao = mockk<AssetsDao>()
    private val priceAlertsDao = mockk<PriceAlertsDao>()

    private fun service() = StreamSubscriptionService(
        assetsDao = assetsDao,
        priceAlertsDao = priceAlertsDao,
    )

    @Test
    fun `setupAssets enqueues subscribe message`() = runTest {
        coEvery { assetsDao.getAssetsPriceUpdate("wallet-1") } returns listOf("bitcoin")
        every { priceAlertsDao.getAlerts() } returns flowOf(emptyList())
        val service = service()

        service.setupAssets(WalletId("wallet-1"))

        val subscribe = service.messages.receive() as StreamMessage.SubscribePrices
        assertEquals(listOf(Chain.Bitcoin), subscribe.data.assets.map { it.chain })
    }

    @Test
    fun `messages sent before consumer attaches are buffered`() = runTest {
        coEvery { assetsDao.getAssetsPriceUpdate("wallet-1") } returns listOf("bitcoin")
        every { priceAlertsDao.getAlerts() } returns flowOf(emptyList())
        val service = service()

        service.setupAssets(WalletId("wallet-1"))
        service.addAssetIds(listOf(AssetId(Chain.Ethereum)))

        val subscribe = service.messages.receive() as StreamMessage.SubscribePrices
        val add = service.messages.receive() as StreamMessage.AddPrices
        assertEquals(listOf(Chain.Bitcoin), subscribe.data.assets.map { it.chain })
        assertEquals(listOf(Chain.Ethereum), add.data.assets.map { it.chain })
    }

    @Test
    fun `addAssetIds skips already subscribed ids`() = runTest {
        every { priceAlertsDao.getAlerts() } returns flowOf(emptyList())
        val service = service()

        val id = AssetId(Chain.Ethereum)
        service.addAssetIds(listOf(id))
        service.addAssetIds(listOf(id))

        service.messages.receive() as StreamMessage.AddPrices
        assertEquals(null, service.messages.tryReceive().getOrNull())
    }
}
