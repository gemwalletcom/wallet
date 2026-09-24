package com.gemwallet.android.features.settings.networks.viewmodels

import androidx.lifecycle.viewModelScope
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemLatencyStatus
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemServiceStatusInterface
import uniffi.gemstone.GemServiceStatusSession
import uniffi.gemstone.GemServiceStatusTarget

@OptIn(ExperimentalCoroutinesApi::class)
class ServiceStatusViewModelTest {
    private val dispatcher = StandardTestDispatcher()
    private val viewModels = mutableListOf<ServiceStatusViewModel>()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        viewModels.forEach { it.viewModelScope.cancel() }
        Dispatchers.resetMain()
    }

    @Test
    fun `a finished check shows while a slower one keeps loading`() = runTest(dispatcher) {
        val stream = CompletableDeferred<GemLatencyStatus>()
        val service = mockk<GemServiceStatusInterface> {
            every { newSession() } answers { GemServiceStatusSession(emptyMap(), GemLatencyStatus.Loading) }
            coEvery { status(GemServiceStatusTarget.Stream) } coAnswers { stream.await() }
            coEvery { status(match { it is GemServiceStatusTarget.Endpoint }) } returns GemLatencyStatus.Error
        }
        val viewModel = ServiceStatusViewModel(service).also(viewModels::add)

        viewModel.fetch()
        advanceUntilIdle()

        assertEquals(
            listOf(GemLatencyStatus.Error, GemLatencyStatus.Loading, GemLatencyStatus.Error, GemLatencyStatus.Error, GemLatencyStatus.Error),
            viewModel.statuses(),
        )

        stream.complete(GemLatencyStatus.Error)
        advanceUntilIdle()

        assertEquals(List(5) { GemLatencyStatus.Error }, viewModel.statuses())
    }

    private fun ServiceStatusViewModel.statuses(): List<GemLatencyStatus> = sections.value.flatMap { it.rows }.map { (it as GemListRow.Latency).status }
}
