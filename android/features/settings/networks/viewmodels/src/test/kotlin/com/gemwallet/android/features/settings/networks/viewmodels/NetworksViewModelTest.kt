package com.gemwallet.android.features.settings.networks.viewmodels

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
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemChainSettingsServiceInterface
import uniffi.gemstone.GemChainSettingsSection
import uniffi.gemstone.GemNodeListSession
import uniffi.gemstone.GemNodeRow
import uniffi.gemstone.GemNodeRowTitle
import uniffi.gemstone.GemNodeSelection
import uniffi.gemstone.GemNodeStatusState
import uniffi.gemstone.Latency
import uniffi.gemstone.LatencyType

@OptIn(ExperimentalCoroutinesApi::class)
class NetworksViewModelTest {

    private val dispatcher = StandardTestDispatcher()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    private fun node(url: String) = GemNodeSelection(url = url, host = url, isSelected = false, gemNodeFlag = null)

    private fun reachable(block: ULong) = GemNodeStatusState.Result(block, Latency(LatencyType.FAST, 10.0))

    private fun service(
        nodesByCall: List<List<GemNodeSelection>>,
        statuses: Map<String, GemNodeStatusState> = emptyMap(),
    ): GemChainSettingsServiceInterface {
        var call = 0
        return mockk(relaxed = true) {
            every { sections() } returns listOf(GemChainSettingsSection.NODES, GemChainSettingsSection.EXPLORER)
            every { chains(any()) } returns listOf(Chain.Ethereum.string)
            every { explorerRows(any()) } returns emptyList()
            every { newNodeListSession(any()) } answers { GemNodeListSession(firstArg(), emptyList(), emptyMap()) }
            every { nodeRows(any(), any(), any()) } answers {
                secondArg<List<GemNodeSelection>>().map { node ->
                    val status = thirdArg<Map<String, GemNodeStatusState>>()[node.url] ?: GemNodeStatusState.Loading
                    GemNodeRow(
                        node = node,
                        title = GemNodeRowTitle.Host(node.host),
                        subtitle = status.subtitle(),
                        latencyStatus = status.latencyStatus(),
                        canDelete = true,
                    )
                }
            }
            coEvery { nodes(any()) } answers { nodesByCall.getOrElse(call) { nodesByCall.last() }.also { call++ } }
            coEvery { nodeStatus(any(), any()) } answers { statuses[secondArg<String>()] ?: GemNodeStatusState.Error }
        }
    }

    @Test
    fun `selecting a chain loads its nodes and a status for each`() = runTest(dispatcher) {
        val service = service(
            nodesByCall = listOf(listOf(node("a"), node("b"))),
            statuses = mapOf("a" to reachable(10UL), "b" to reachable(11UL)),
        )
        val viewModel = NetworksViewModel(service)

        viewModel.onSelectedChain(Chain.Ethereum)
        advanceUntilIdle()

        assertEquals(Chain.Ethereum, viewModel.uiState.value.chain)
        assertTrue(viewModel.uiState.value.selectChain.not())
    }

    @Test
    fun `deleting a node drops the status it had`() = runTest(dispatcher) {
        val service = service(
            nodesByCall = listOf(listOf(node("a"), node("b")), listOf(node("a"))),
            statuses = mapOf("a" to reachable(10UL), "b" to reachable(11UL)),
        )
        val viewModel = NetworksViewModel(service)
        viewModel.onSelectedChain(Chain.Ethereum)
        advanceUntilIdle()

        viewModel.onDeleteNode("b")
        advanceUntilIdle()

        assertEquals(listOf("a"), sessionUrls(viewModel))
    }

    private fun sessionUrls(viewModel: NetworksViewModel): List<String> {
        val rows = viewModel.uiState.value
        return rows.nodeRows.map { it.node.url }
    }
}
