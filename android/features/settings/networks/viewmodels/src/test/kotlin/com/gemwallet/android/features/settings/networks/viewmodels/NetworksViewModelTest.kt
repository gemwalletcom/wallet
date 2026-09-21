package com.gemwallet.android.features.settings.networks.viewmodels

import android.content.Context
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.features.settings.networks.viewmodels.models.NetworkSectionUIModel
import com.gemwallet.android.testkit.mockGemNodeSelection
import com.gemwallet.android.testkit.mockGemNodeStatusState
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
import uniffi.gemstone.GemChainSettingsServiceInterface
import uniffi.gemstone.GemNodeListSession
import uniffi.gemstone.GemNodeSelection
import uniffi.gemstone.GemNodeStatusState

@OptIn(ExperimentalCoroutinesApi::class)
class NetworksViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<NetworksViewModel>()

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
            every { chains(any()) } returns listOf(Chain.Ethereum.string)
            every { explorerRows(any()) } returns emptyList()
            every { newNodeListSession(any()) } answers { GemNodeListSession(firstArg(), emptyList(), emptyMap()) }
            coEvery { nodes(any()) } answers { nodesByCall.getOrElse(call) { nodesByCall.last() }.also { call++ } }
            coEvery { nodeStatus(any(), any()) } answers { statuses[secondArg<String>()] ?: GemNodeStatusState.Error }
        }
    }

    @Test
    fun `selecting a chain loads its nodes and a status for each`() = runTest(dispatcher) {
        val service = service(
            nodesByCall = listOf(listOf(mockGemNodeSelection("a"), mockGemNodeSelection("b"))),
            statuses = mapOf("a" to mockGemNodeStatusState(10UL), "b" to mockGemNodeStatusState(11UL)),
        )
        val viewModel = NetworksViewModel(
            service,
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
            nodesByCall = listOf(listOf(mockGemNodeSelection("a"), mockGemNodeSelection("b")), listOf(mockGemNodeSelection("a"))),
            statuses = mapOf("a" to mockGemNodeStatusState(10UL), "b" to mockGemNodeStatusState(11UL)),
        )
        val viewModel = NetworksViewModel(
            service,
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

    private fun sessionUrls(viewModel: NetworksViewModel): List<String> {
        val rows = viewModel.uiState.value
        return rows.sections.filterIsInstance<NetworkSectionUIModel.Nodes>().flatMap { it.rows }.map { it.url }
    }
}
