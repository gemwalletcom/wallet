package com.gemwallet.android.features.settings.viewmodels.chain_settings

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockGemNodeSelection
import com.gemwallet.android.testkit.mockLatency
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemChainSettingsSection
import uniffi.gemstone.GemChainSettingsServiceInterface
import uniffi.gemstone.GemNodeListSession
import uniffi.gemstone.GemNodeSelection
import uniffi.gemstone.GemNodeStatusState

@OptIn(ExperimentalCoroutinesApi::class)
class ChainSettingsViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<ChainSettingsViewModel>()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
    }

    private fun service(nodesByCall: List<List<GemNodeSelection>>, statuses: Map<String, GemNodeStatusState> = emptyMap()): GemChainSettingsServiceInterface {
        var call = 0
        return mockk(relaxed = true) {
            every { explorerRows(any()) } returns emptyList()
            every { newNodeListSession(any()) } answers { GemNodeListSession(firstArg(), emptyList(), emptyMap()) }
            coEvery { nodes(any()) } answers { nodesByCall.getOrElse(call) { nodesByCall.last() }.also { call++ } }
            coEvery { nodeStatus(any(), any()) } answers { statuses[secondArg<String>()] ?: GemNodeStatusState.Error }
        }
    }

    private fun viewModel(service: GemChainSettingsServiceInterface, chain: Chain = Chain.Ethereum) = ChainSettingsViewModel(
        service,
        SavedStateHandle(mapOf(RouteArgument.Chain.key to chain.string)),
        dispatcher,
        mockk<Context> {
            every { getString(any()) } returns "Error"
            every { getString(any(), *anyVararg()) } returns "Error"
        },
    ).also { models.add(it) }

    @Test
    fun `opening a chain loads the route chain's nodes and a status for each`() = runTest(dispatcher) {
        val service = service(
            nodesByCall = listOf(listOf(mockGemNodeSelection(url = "a", host = "a"), mockGemNodeSelection(url = "b", host = "b"))),
            statuses = mapOf(
                "a" to GemNodeStatusState.Result(latestBlockNumber = 10UL, latency = mockLatency(value = 10.0).toGem()),
                "b" to GemNodeStatusState.Result(latestBlockNumber = 11UL, latency = mockLatency(value = 10.0).toGem()),
            ),
        )
        val viewModel = viewModel(service, Chain.Solana)
        advanceUntilIdle()

        assertEquals(Chain.Solana, viewModel.uiState.value.chain)
        coVerify(exactly = 1) { service.nodes(Chain.Solana.string) }
        coVerify(exactly = 1) { service.nodeStatus(Chain.Solana.string, "a") }
        coVerify(exactly = 1) { service.nodeStatus(Chain.Solana.string, "b") }
        assertEquals(listOf("a", "b"), sessionUrls(viewModel))
    }

    @Test
    fun `deleting a node drops the status it had`() = runTest(dispatcher) {
        val service = service(
            nodesByCall = listOf(listOf(mockGemNodeSelection(url = "a", host = "a"), mockGemNodeSelection(url = "b", host = "b")), listOf(mockGemNodeSelection(url = "a", host = "a"))),
            statuses = mapOf(
                "a" to GemNodeStatusState.Result(latestBlockNumber = 10UL, latency = mockLatency(value = 10.0).toGem()),
                "b" to GemNodeStatusState.Result(latestBlockNumber = 11UL, latency = mockLatency(value = 10.0).toGem()),
            ),
        )
        val viewModel = viewModel(service)
        advanceUntilIdle()

        viewModel.onDeleteNode("b")
        advanceUntilIdle()

        assertEquals(listOf("a"), sessionUrls(viewModel))
    }

    private fun sessionUrls(viewModel: ChainSettingsViewModel): List<String> {
        val rows = viewModel.uiState.value
        return rows.sections.filterIsInstance<GemChainSettingsSection.Nodes>().flatMap { it.rows }.map { it.node.url }
    }
}
