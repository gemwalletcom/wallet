package com.gemwallet.android.features.perpetual.viewmodels

import kotlinx.coroutines.flow.first
import com.gemwallet.android.testkit.mockAsset
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.PerpetualId
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDataAggregate
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPositions
import com.gemwallet.android.application.perpetual.cases.GetPerpetuals
import com.gemwallet.android.application.perpetual.cases.PerpetualObserver
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.wallet.core.primitives.Chain
import androidx.lifecycle.viewModelScope
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.cancel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemMarketsRefreshTrigger
import uniffi.gemstone.GemPerpetualServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class PerpetualMarketViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private var model: PerpetualMarketViewModel? = null

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        model?.viewModelScope?.cancel()
        model = null
        Dispatchers.resetMain()
    }

    @Test
    fun `pull to refresh asks core for a user requested markets sync`() = runTest(dispatcher) {
        val trigger = CompletableDeferred<GemMarketsRefreshTrigger>()
        val service = mockk<GemPerpetualServiceInterface>()
        coEvery { service.refresh(any()) } answers { trigger.complete(firstArg()); emptyList() }

        viewModel(service).onRefresh()

        assertEquals(GemMarketsRefreshTrigger.USER_REQUESTED, trigger.await())
    }

    @Test
    fun `opening the screen asks core for a scheduled refresh, not positions alone`() = runTest(dispatcher) {
        val trigger = CompletableDeferred<GemMarketsRefreshTrigger>()
        val service = mockk<GemPerpetualServiceInterface>()
        coEvery { service.refresh(any()) } answers { trigger.complete(firstArg()); emptyList() }

        viewModel(service).fetch()

        assertEquals(GemMarketsRefreshTrigger.SCHEDULED, trigger.await())
    }

    private fun viewModel(
        service: GemPerpetualServiceInterface,
        positions: List<PerpetualPositionDataAggregate> = emptyList(),
    ): PerpetualMarketViewModel {
        val getPerpetuals = mockk<GetPerpetuals>()
        every { getPerpetuals.getPerpetuals(any<Flow<String?>>()) } returns flowOf(emptyList())
        val getPositions = mockk<GetPerpetualPositions>()
        every { getPositions.getPerpetualPositions() } returns flowOf(positions)
        val getBalance = mockk<GetPerpetualBalance>()
        every { getBalance.getDisplayBalance() } returns emptyFlow()
        every { getBalance.getBalance() } returns emptyFlow()
        val recentAssetsService = mockk<RecentAssetsService>()
        every { recentAssetsService.getRecentAssets(any()) } returns flowOf(emptyList())
        val perpetualObserver = mockk<PerpetualObserver>()

        return PerpetualMarketViewModel(
            getPerpetuals = getPerpetuals,
            getPositions = getPositions,
            getBalance = getBalance,
            recentAssetsService = recentAssetsService,
            service = service,
            perpetualObserver = perpetualObserver,
            ioDispatcher = dispatcher,
            context = mockk(relaxed = true),
            connectionStatusObserver = mockk(relaxed = true),
        ).also { model = it }
    }

    @Test
    fun `a position is found by its perpetual symbol like on iOS`() = runTest(dispatcher) {
        val position = mockk<PerpetualPositionDataAggregate>(relaxed = true) {
            every { title } returns "Bitcoin"
            every { perpetualId } returns PerpetualId(PerpetualProvider.Hypercore, "BTC-USD")
            every { asset } returns mockAsset(chain = Chain.Bitcoin, name = "Bitcoin", symbol = "BTC")
        }
        val viewModel = viewModel(mockk(relaxed = true), positions = listOf(position))
        viewModel.setQuery("btc-usd")
        assertEquals(listOf(position), viewModel.positions.first { it.isNotEmpty() })
        viewModel.setQuery("zzz")
        assertEquals(emptyList<PerpetualPositionDataAggregate>(), viewModel.positions.first { it.isEmpty() })
    }
}
