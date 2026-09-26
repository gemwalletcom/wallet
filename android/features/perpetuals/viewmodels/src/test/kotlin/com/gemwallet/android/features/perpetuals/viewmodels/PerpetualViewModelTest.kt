package com.gemwallet.android.features.perpetuals.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.perpetual.cases.PerpetualObserver
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.data.services.store.queries.PerpetualPositionsQuery
import com.gemwallet.android.data.services.store.queries.PerpetualQuery
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockPerpetual
import com.gemwallet.android.testkit.mockPerpetualData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemCandleResult
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemPerpetualDetailsServiceInterface
import uniffi.gemstone.GemPerpetualSubscription

@OptIn(ExperimentalCoroutinesApi::class)
class PerpetualViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<PerpetualViewModel>()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
        Dispatchers.resetMain()
    }

    private val asset = mockAsset()

    private fun perpetualData(): PerpetualData = mockPerpetualData(perpetual = mockPerpetual(), asset = asset)

    private fun viewModel(
        service: GemPerpetualDetailsServiceInterface = mockk(relaxed = true) {
            every { chartPeriod() } returns uniffi.gemstone.ChartPeriod.DAY
            coEvery { candles(any()) } answers { GemCandleResult(request = firstArg(), state = GemLoadState.Data, candles = emptyList()) }
        },
        data: PerpetualData? = null,
        perpetuals: Flow<PerpetualData?> = flowOf(data),
        positions: PerpetualPositionsQuery = mockk(relaxed = true),
        observer: PerpetualObserver = mockk(relaxed = true) {
            every { chartUpdates } returns emptyFlow()
        },
    ): PerpetualViewModel {
        val session: GetSession = mockk {
            every { this@mockk.invoke() } returns MutableStateFlow(mockSession())
        }
        val perpetual: PerpetualQuery = mockk {
            every { this@mockk(any<AssetId>()) } returns perpetuals
        }
        val transactions: GetTransactions = mockk {
            every { getTransactions(any(), any()) } returns emptyFlow()
            every { stored(any(), any()) } returns emptyList()
        }
        return PerpetualViewModel(
            perpetual,
            positions,
            transactions,
            observer,
            service,
            session,
            SavedStateHandle(mapOf(RouteArgument.AssetId.key to asset.id.toIdentifier())),
            dispatcher,
            mockk(relaxed = true),
        ).also { models.add(it) }
    }

    @Test
    fun `a price-only market update keeps the subscriptions and the position query`() = runTest(dispatcher) {
        val service: GemPerpetualDetailsServiceInterface = mockk(relaxed = true) {
            every { chartPeriod() } returns uniffi.gemstone.ChartPeriod.DAY
            coEvery { candles(any()) } answers { GemCandleResult(request = firstArg(), state = GemLoadState.Data, candles = emptyList()) }
            every { candleSubscription(any(), any()) } answers { GemPerpetualSubscription.Candle(firstArg<uniffi.gemstone.Perpetual>().name, "1d") }
            every { marketSubscription(any()) } answers { GemPerpetualSubscription.MarketData(firstArg<uniffi.gemstone.Perpetual>().name) }
        }
        val markets = MutableStateFlow<PerpetualData?>(mockPerpetualData(perpetual = mockPerpetual(price = 1.0), asset = asset))
        val positions: PerpetualPositionsQuery = mockk(relaxed = true)
        val observer: PerpetualObserver = mockk(relaxed = true) {
            every { chartUpdates } returns emptyFlow()
        }
        val model = viewModel(service = service, perpetuals = markets, positions = positions, observer = observer)
        model.onScreenEnter()
        advanceUntilIdle()

        markets.value = mockPerpetualData(perpetual = mockPerpetual(price = 2.0), asset = asset)
        advanceUntilIdle()

        verify(exactly = 2) { observer.subscribe(any()) }
        verify(exactly = 0) { observer.unsubscribe(any()) }
        verify(exactly = 1) { positions(any(), any<PerpetualId>()) }
    }

    @Test
    fun `the chart period is remembered in Core`() = runTest(dispatcher) {
        val service: GemPerpetualDetailsServiceInterface = mockk(relaxed = true) {
            every { chartPeriod() } returns uniffi.gemstone.ChartPeriod.DAY
            coEvery { candles(any()) } answers { GemCandleResult(request = firstArg(), state = GemLoadState.Data, candles = emptyList()) }
        }
        val model = viewModel(service = service)

        model.period(ChartPeriod.Week)
        advanceUntilIdle()

        assertEquals(ChartPeriod.Week, model.period.first { it == ChartPeriod.Week })
        coVerify { service.setChartPeriod(uniffi.gemstone.ChartPeriod.WEEK) }
    }

    @Test
    fun `refreshing asks Core for the stored data again`() = runTest(dispatcher) {
        val service: GemPerpetualDetailsServiceInterface = mockk(relaxed = true) {
            every { chartPeriod() } returns uniffi.gemstone.ChartPeriod.DAY
            coEvery { candles(any()) } answers { GemCandleResult(request = firstArg(), state = GemLoadState.Data, candles = emptyList()) }
        }
        val model = viewModel(service = service)
        advanceUntilIdle()
        coVerify(exactly = 0) { service.refresh(any()) }

        model.refresh()
        advanceUntilIdle()

        assertFalse("the spinner stops once the answer lands", model.isRefreshing.value)
        coVerify(exactly = 1) { service.refresh(asset.id.toIdentifier()) }

        model.refreshPerpetual()
        advanceUntilIdle()
        coVerify(exactly = 2) { service.refresh(asset.id.toIdentifier()) }
    }

    @Test
    fun `closing a position without a perpetual does nothing`() = runTest(dispatcher) {
        val service: GemPerpetualDetailsServiceInterface = mockk(relaxed = true) {
            every { chartPeriod() } returns uniffi.gemstone.ChartPeriod.DAY
            coEvery { candles(any()) } answers { GemCandleResult(request = firstArg(), state = GemLoadState.Data, candles = emptyList()) }
        }
        val model = viewModel(service = service)
        val confirm: ConfirmTransactionAction = mockk(relaxed = true)

        model.closePosition(confirm)

        assertNull(model.error.value)
        coVerify(exactly = 0) { service.closeTransfer(any(), any(), any()) }
    }

    @Test
    fun `a failed close reads the error instead of crashing the screen`() = runTest(dispatcher) {
        val service: GemPerpetualDetailsServiceInterface = mockk(relaxed = true) {
            every { chartPeriod() } returns uniffi.gemstone.ChartPeriod.DAY
            every { closeTransfer(any(), any(), any()) } throws IllegalStateException("no position")
        }
        val model = viewModel(service = service, data = perpetualData())
        val confirm: ConfirmTransactionAction = mockk(relaxed = true)
        model.perpetual.first { it != null }

        model.closePosition(confirm)

        assertEquals("no position", model.error.value)
        verify(exactly = 0) { confirm(any()) }
        model.clearError()
        assertNull(model.error.value)
    }

    @Test
    fun `a failed position action reads the error`() = runTest(dispatcher) {
        val service: GemPerpetualDetailsServiceInterface = mockk(relaxed = true) {
            every { chartPeriod() } returns uniffi.gemstone.ChartPeriod.DAY
            every { positionAction(any(), any(), any(), any()) } throws IllegalStateException("no market")
        }
        val model = viewModel(service = service, data = perpetualData())
        val amount: AmountTransactionAction = mockk(relaxed = true)
        model.perpetual.first { it != null }

        model.increasePosition(amount)

        assertEquals("no market", model.error.value)
        verify(exactly = 0) { amount(any()) }
    }
}
