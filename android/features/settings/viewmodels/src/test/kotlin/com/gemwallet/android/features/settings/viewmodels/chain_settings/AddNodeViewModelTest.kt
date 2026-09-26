package com.gemwallet.android.features.settings.viewmodels.chain_settings

import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemAddNodeException
import uniffi.gemstone.GemAddNodePhase
import uniffi.gemstone.GemAddNodeSession
import uniffi.gemstone.GemChainSettingsServiceInterface
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemNodeCheck
import uniffi.gemstone.GemNodeCheckRow
import uniffi.gemstone.GemNodeSyncState
import uniffi.gemstone.Latency
import uniffi.gemstone.LatencyType

@OptIn(ExperimentalCoroutinesApi::class)
class AddNodeViewModelTest {

    private val dispatcher = StandardTestDispatcher()

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
        coEvery { checkNode(any(), any()) } answers { answer() }
        coEvery { addNode(any(), any()) } returns Unit
    }

    @Test
    fun `an untouched form is idle`() = runTest(dispatcher) {
        val viewModel = AddNodeViewModel(service { check }, dispatcher)

        viewModel.init(Chain.Ethereum)
        advanceUntilIdle()

        assertEquals(GemAddNodePhase.Idle, viewModel.viewState.value?.phase)
        assertEquals(false, viewModel.viewState.value?.canImport)
    }

    @Test
    fun `a node that answers is ready to import`() = runTest(dispatcher) {
        val viewModel = AddNodeViewModel(service { check }, dispatcher)
        viewModel.init(Chain.Ethereum)

        viewModel.url.value = "https://node"
        viewModel.onUrlChange()
        advanceUntilIdle()

        val state = viewModel.viewState.value!!
        assertEquals(true, state.canImport)
        assertEquals(true, state.showsWarning)
        val rows = (state.phase as GemAddNodePhase.Ready).rows
        assertEquals(GemNodeCheckRow.ChainId("1"), rows.first())
        assertEquals(GemNodeCheckRow.InSync(GemNodeSyncState.IN_SYNC), rows[1])
    }

    @Test
    fun `a rejected network id is reported and cannot be imported`() = runTest(dispatcher) {
        val viewModel = AddNodeViewModel(service { throw GemAddNodeException.InvalidNetworkId() }, dispatcher)
        viewModel.init(Chain.Ethereum)

        viewModel.url.value = "https://node"
        viewModel.onUrlChange()
        advanceUntilIdle()

        assertEquals(GemAddNodePhase.Failed(GemErrorText.InvalidNetworkId), viewModel.viewState.value?.phase)
        assertEquals(false, viewModel.viewState.value?.canImport)
    }

    @Test
    fun `clearing the field stops asking`() = runTest(dispatcher) {
        val viewModel = AddNodeViewModel(service { check }, dispatcher)
        viewModel.init(Chain.Ethereum)

        viewModel.url.value = ""
        viewModel.onUrlChange()
        advanceUntilIdle()

        assertEquals(GemAddNodePhase.Idle, viewModel.viewState.value?.phase)
    }

    @Test
    fun `a submission in flight ignores repeated taps and keeps a newer entry`() = runTest(dispatcher) {
        val added = CompletableDeferred<Unit>()
        val service = service { check }
        coEvery { service.addNode(any(), any()) } coAnswers { added.await() }
        val viewModel = AddNodeViewModel(service, dispatcher)
        viewModel.init(Chain.Ethereum)
        viewModel.url.value = "https://node"
        viewModel.onUrlChange()
        advanceUntilIdle()
        var navigations = 0

        viewModel.addUrl { navigations++ }
        viewModel.addUrl { navigations++ }
        runCurrent()
        viewModel.url.value = "https://other"
        viewModel.onUrlChange()
        added.complete(Unit)
        advanceUntilIdle()

        coVerify(exactly = 1) { service.addNode(any(), any()) }
        assertEquals(1, navigations)
        assertEquals("https://other", viewModel.url.value)
    }
}
