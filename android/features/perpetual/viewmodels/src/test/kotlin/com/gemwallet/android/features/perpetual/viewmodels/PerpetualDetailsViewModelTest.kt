package com.gemwallet.android.features.perpetual.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.perpetual.cases.BuildPerpetualParams
import com.gemwallet.android.application.perpetual.cases.GetPerpetual
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPosition
import com.gemwallet.android.application.perpetual.cases.PerpetualObserver
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.ChartPeriod
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemPerpetualDetailsServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class PerpetualDetailsViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<PerpetualDetailsViewModel>()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
        Dispatchers.resetMain()
    }

    private val asset = mockAsset()

    private fun viewModel(
        service: GemPerpetualDetailsServiceInterface = mockk(relaxed = true) {
            every { chartPeriod() } returns uniffi.gemstone.ChartPeriod.DAY
        },
        params: BuildPerpetualParams = mockk(relaxed = true),
    ): PerpetualDetailsViewModel {
        val session: GetSession = mockk {
            every { this@mockk.invoke() } returns MutableStateFlow(mockSession())
        }
        val perpetual: GetPerpetual = mockk {
            every { getPerpetualByAssetId(any()) } returns flowOf(null)
        }
        val positions: GetPerpetualPosition = mockk(relaxed = true)
        val transactions: GetTransactions = mockk {
            every { getTransactions(any()) } returns emptyFlow()
        }
        val observer: PerpetualObserver = mockk(relaxed = true) {
            every { chartUpdates } returns emptyFlow()
        }
        return PerpetualDetailsViewModel(
            perpetual,
            positions,
            transactions,
            params,
            observer,
            service,
            session,
            SavedStateHandle(mapOf(RouteArgument.AssetId.key to asset.id.toIdentifier())),
            mockk(relaxed = true),
        ).also { models.add(it) }
    }

    @Test
    fun `the chart period is remembered in Core`() = runTest(dispatcher) {
        val service: GemPerpetualDetailsServiceInterface = mockk(relaxed = true) {
            every { chartPeriod() } returns uniffi.gemstone.ChartPeriod.DAY
        }
        val model = viewModel(service = service)

        model.period(ChartPeriod.Week)

        assertEquals(ChartPeriod.Week, model.period.first { it == ChartPeriod.Week })
        coVerify { service.setChartPeriod(uniffi.gemstone.ChartPeriod.WEEK) }
    }

    @Test
    fun `refreshing shows the spinner and asks Core to sync the positions`() = runTest(dispatcher) {
        val service: GemPerpetualDetailsServiceInterface = mockk(relaxed = true) {
            every { chartPeriod() } returns uniffi.gemstone.ChartPeriod.DAY
        }
        val model = viewModel(service = service)

        model.refresh()

        assertTrue(model.isRefreshing.value)
        coVerify { service.syncPositions() }
    }

    @Test
    fun `closing a position without a perpetual does nothing`() = runTest(dispatcher) {
        val params: BuildPerpetualParams = mockk(relaxed = true)
        val model = viewModel(params = params)
        val confirm: ConfirmTransactionAction = mockk(relaxed = true)

        model.closePosition(confirm)

        assertNull(model.error.value)
        coVerify(exactly = 0) { params.close(any()) }
    }

    @Test
    fun `an error is shown until it is cleared`() = runTest(dispatcher) {
        val model = viewModel()

        assertNull(model.error.value)
        model.clearError()
        assertNull(model.error.value)
    }
}
