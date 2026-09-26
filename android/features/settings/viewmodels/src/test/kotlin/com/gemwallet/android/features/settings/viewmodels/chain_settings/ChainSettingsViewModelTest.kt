package com.gemwallet.android.features.settings.viewmodels.chain_settings

import android.content.Context
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.settings.viewmodels.chain_settings.models.ChainSettingsSectionUIModel
import com.gemwallet.android.testkit.mockGemNodeSelection
import com.gemwallet.android.testkit.mockLatency
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
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
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemChainService
import uniffi.gemstone.GemChainSettingsServiceInterface
import uniffi.gemstone.GemNodeListSession
import uniffi.gemstone.GemNodeSelection
import uniffi.gemstone.GemNodeStatusState

@OptIn(ExperimentalCoroutinesApi::class)
class ChainSettingsViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<ChainSettingsViewModel>()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
        Dispatchers.resetMain()
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

    @Test
    fun `selecting a chain loads its nodes and a status for each`() = runTest(dispatcher) {
        val service = service(
            nodesByCall = listOf(listOf(mockGemNodeSelection(url = "a", host = "a"), mockGemNodeSelection(url = "b", host = "b"))),
            statuses = mapOf(
                "a" to GemNodeStatusState.Result(latestBlockNumber = 10UL, latency = mockLatency(value = 10.0).toGem()),
                "b" to GemNodeStatusState.Result(latestBlockNumber = 11UL, latency = mockLatency(value = 10.0).toGem()),
            ),
        )
        val viewModel = ChainSettingsViewModel(
            service,
            GemChainService(),
            dispatcher,
            mockk<Context> {
                every { getString(any()) } returns "Error"
                every { getString(any(), *anyVararg()) } returns "Error"
            },
        ).also { models.add(it) }

        viewModel.onSelectedChain(Chain.Ethereum)
        advanceUntilIdle()

        assertEquals(Chain.Ethereum, viewModel.uiState.value.chain)
        assertTrue(viewModel.uiState.value.selectChain.not())
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
        val viewModel = ChainSettingsViewModel(
            service,
            GemChainService(),
            dispatcher,
            mockk<Context> {
                every { getString(any()) } returns "Error"
                every { getString(any(), *anyVararg()) } returns "Error"
            },
        ).also { models.add(it) }
        viewModel.onSelectedChain(Chain.Ethereum)
        advanceUntilIdle()

        viewModel.onDeleteNode("b")
        advanceUntilIdle()

        assertEquals(listOf("a"), sessionUrls(viewModel))
    }

    private fun sessionUrls(viewModel: ChainSettingsViewModel): List<String> {
        val rows = viewModel.uiState.value
        return rows.sections.filterIsInstance<ChainSettingsSectionUIModel.Nodes>().flatMap { it.rows }.map { it.row.node.url }
    }
}
