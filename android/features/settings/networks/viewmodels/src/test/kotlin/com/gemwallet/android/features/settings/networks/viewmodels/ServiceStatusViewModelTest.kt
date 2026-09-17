package com.gemwallet.android.features.settings.networks.viewmodels

import android.content.Context
import com.gemwallet.android.ui.components.list_item.ListItemTagType
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
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
import uniffi.gemstone.GemServiceEndpoint
import uniffi.gemstone.GemServiceEndpointType
import uniffi.gemstone.GemServiceStatusInterface
import uniffi.gemstone.Latency
import uniffi.gemstone.LatencyType

@OptIn(ExperimentalCoroutinesApi::class)
class ServiceStatusViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val context = mockk<Context> {
        every { getString(any()) } returns "Error"
        every { getString(any(), *anyVararg()) } returns "10 ms"
    }

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    private val api = GemServiceEndpoint(endpointType = GemServiceEndpointType.API, host = "api", url = "api", flag = "🇺🇸")
    private val node = GemServiceEndpoint(endpointType = GemServiceEndpointType.GEM_NODE, host = "node", url = "node", flag = "🇯🇵")

    private fun service(statuses: Map<String, GemLatencyStatus>): GemServiceStatusInterface = mockk(relaxed = true) {
        every { getEndpoints() } returns listOf(api, node)
        coEvery { getEndpointStatus(any()) } answers { statuses[firstArg<String>()] ?: GemLatencyStatus.Error }
    }

    @Test
    fun `every endpoint starts loading`() = runTest(dispatcher) {
        val viewModel = ServiceStatusViewModel(service(emptyMap()), context)

        assertEquals(listOf("api", "node"), viewModel.uiState.value.rows.map { it.id })
        assertEquals(listOf(ListItemTagType.Progress, ListItemTagType.Progress), viewModel.uiState.value.rows.map { it.model.titleTagType })
    }

    @Test
    fun `each endpoint keeps its own answer`() = runTest(dispatcher) {
        val reachable = GemLatencyStatus.Result(Latency(LatencyType.FAST, 10.0))
        val viewModel = ServiceStatusViewModel(service(mapOf("api" to reachable)), context)

        viewModel.fetch()
        advanceUntilIdle()

        assertEquals(listOf(ListItemTextStyle.Positive, ListItemTextStyle.Negative), viewModel.uiState.value.rows.map { it.model.titleTagStyle })
        assertEquals(listOf("10 ms", "Error"), viewModel.uiState.value.rows.map { it.model.titleTag })
    }
}
