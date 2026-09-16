package com.gemwallet.android.features.settings.networks.viewmodels

import android.content.Context
import com.gemwallet.android.features.settings.networks.viewmodels.models.AddNodeUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ButtonState
import com.wallet.core.primitives.Chain
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
import uniffi.gemstone.GemAddNodeException
import uniffi.gemstone.GemAddNodeSession
import uniffi.gemstone.GemChainSettingsServiceInterface
import uniffi.gemstone.GemNodeCheck
import uniffi.gemstone.Latency
import uniffi.gemstone.LatencyType

@OptIn(ExperimentalCoroutinesApi::class)
class AddNodeViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val context = mockk<Context> {
        every { getString(any()) } returns "Error"
        every { getString(any(), *anyVararg()) } returns "Error"
    }

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    private val check = GemNodeCheck(
        url = "https://node",
        chainId = "1",
        latestBlockNumber = 21_000_000UL,
        isInSync = true,
        latency = Latency(LatencyType.FAST, 12.0),
    )

    private fun service(answer: () -> GemNodeCheck): GemChainSettingsServiceInterface = mockk(relaxed = true) {
        every { newAddNodeSession(any()) } answers { GemAddNodeSession(firstArg(), "", null, null, false) }
        every { nodeCheckDebounceMilliseconds() } returns 0UL
        coEvery { checkNode(any(), any()) } answers { answer() }
        coEvery { addNode(any(), any()) } returns Unit
    }

    @Test
    fun `an untouched form is idle`() = runTest(dispatcher) {
        val viewModel = AddNodeViewModel(service { check }, dispatcher, context)

        viewModel.init(Chain.Ethereum)
        advanceUntilIdle()

        assertEquals(AddNodeUIModel(chain = Chain.Ethereum), viewModel.uiModel.value)
    }

    @Test
    fun `a node that answers is ready to import`() = runTest(dispatcher) {
        val viewModel = AddNodeViewModel(service { check }, dispatcher, context)
        viewModel.init(Chain.Ethereum)

        viewModel.url.value = "https://node"
        viewModel.onUrlChange()
        advanceUntilIdle()

        val model = viewModel.uiModel.value
        assertEquals(ButtonState.Enabled, model.buttonState)
        assertEquals("1", model.checks.first { it.title == R.string.nodes_import_node_chain_id }.value)
        assertEquals(true, model.checks.first { it.title == R.string.nodes_import_node_in_sync }.isInSync)
    }

    @Test
    fun `a rejected network id is reported and cannot be imported`() = runTest(dispatcher) {
        val viewModel = AddNodeViewModel(service { throw GemAddNodeException.InvalidNetworkId() }, dispatcher, context)
        viewModel.init(Chain.Ethereum)

        viewModel.url.value = "https://node"
        viewModel.onUrlChange()
        advanceUntilIdle()

        assertEquals("Error", viewModel.uiModel.value.errorText)
        assertEquals(ButtonState.Disabled, viewModel.uiModel.value.buttonState)
    }

    @Test
    fun `clearing the field stops asking`() = runTest(dispatcher) {
        val viewModel = AddNodeViewModel(service { check }, dispatcher, context)
        viewModel.init(Chain.Ethereum)

        viewModel.url.value = ""
        viewModel.onUrlChange()
        advanceUntilIdle()

        assertEquals(AddNodeUIModel(chain = Chain.Ethereum), viewModel.uiModel.value)
    }
}
