package com.gemwallet.android.data.repositories.nodes

import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.data.service.store.database.NodesDao
import com.gemwallet.android.data.service.store.database.entities.DbNode
import com.wallet.core.primitives.Chain
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import io.mockk.verify
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.Config
import uniffi.gemstone.NodePriority

class NodesRepositoryTest {

    private val nodesDao = mockk<NodesDao>(relaxed = true)
    private val configStore = mockk<ConfigStore>()
    private val config = mockk<Config>()

    @After
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun getCurrentBlockExplorer_fallsBackToSupportedExplorerWhenStoredValueIsInvalid() = runTest {
        every { configStore.getString("current_explorer", Chain.Near.string) } returns "NEAR Intents"
        every { config.getBlockExplorers(Chain.Near.string) } returns listOf("Near")
        every { config.getNodes() } returns emptyMap()

        val subject = NodesRepository(
            nodesDao = nodesDao,
            configStore = configStore,
            config = config,
        )

        assertEquals("Near", subject.getCurrentBlockExplorer(Chain.Near))
    }

    @Test
    fun addNode_insertsNode() = runTest {
        every { config.getNodes() } returns emptyMap()
        val subject = NodesRepository(
            nodesDao = nodesDao,
            configStore = configStore,
            config = config,
        )

        subject.addNode(Chain.Ethereum, "https://ethereum.example")

        coVerify {
            nodesDao.addNodes(match<List<DbNode>> { nodes ->
                nodes.single().chain == Chain.Ethereum && nodes.single().url == "https://ethereum.example"
            })
        }
    }

    @Test
    fun getNodes_mergesConfigNodesWithoutWritingThem() = runTest {
        val configNode = mockk<uniffi.gemstone.Node>(relaxed = true)
        every { configNode.url } returns "https://config.example"
        every { configNode.priority } returns NodePriority.HIGH
        every { config.getNodes() } returns mapOf(Chain.Ethereum.string to listOf(configNode))
        every { nodesDao.getNodes(Chain.Ethereum) } returns flowOf(emptyList())
        val subject = NodesRepository(
            nodesDao = nodesDao,
            configStore = configStore,
            config = config,
        )

        val nodes = subject.getNodes(Chain.Ethereum).first()

        assertEquals(true, nodes.any { it.url == "https://config.example" })
        coVerify(exactly = 0) { nodesDao.addNodes(any()) }
    }

    @Test
    fun getNodeUrl_returnsSelectedNodeUrl() {
        every { configStore.getString("usage_node", Chain.Ethereum.string) } returns """
            {"url":"https://custom.example","status":"active","priority":0}
        """.trimIndent()
        val subject = NodesRepository(
            nodesDao = nodesDao,
            configStore = configStore,
            config = config,
        )

        assertEquals("https://custom.example", subject.getNodeUrl(Chain.Ethereum))
    }

    @Test
    fun getNodeUrl_fallsBackToDefaultNodeWithoutPersistingSelection() {
        every { configStore.getString("usage_node", Chain.Ethereum.string) } returns ""
        val subject = NodesRepository(
            nodesDao = nodesDao,
            configStore = configStore,
            config = config,
        )

        assertEquals("https://gemnodes.com/ethereum", subject.getNodeUrl(Chain.Ethereum))
        coVerify(exactly = 0) { nodesDao.addNodes(any()) }
        verify(exactly = 0) { configStore.putString(any(), any(), any()) }
    }
}
