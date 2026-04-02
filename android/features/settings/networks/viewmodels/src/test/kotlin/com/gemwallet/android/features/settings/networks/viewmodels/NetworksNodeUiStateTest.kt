package com.gemwallet.android.features.settings.networks.viewmodels

import com.gemwallet.android.cases.nodes.getGemNode
import com.gemwallet.android.model.NodeStatus
import com.gemwallet.android.features.settings.networks.viewmodels.models.NodeStatusState
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node
import com.wallet.core.primitives.NodeState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class NetworksNodeUiStateTest {

    @Test
    fun `visibleNodeStates removes deleted node entries`() {
        val gemNode = getGemNode(Chain.Bitcoin)
        val remainingNode = Node(
            url = "https://custom.example.com/bitcoin",
            status = NodeState.Active,
            priority = 0,
        )
        val deletedNode = Node(
            url = "https://deleted.example.com/bitcoin",
            status = NodeState.Active,
            priority = 0,
        )
        val nodeStates = mapOf(
            gemNode.url to NodeStatusState.Loading,
            remainingNode.url to NodeStatusState.Error,
            deletedNode.url to NodeStatusState.Result(
                latestBlock = 1UL,
                latency = 20UL,
                chainId = "bitcoin",
            ),
        )

        val visibleStates = visibleNodeStates(
            nodes = listOf(gemNode, remainingNode),
            nodeStates = nodeStates,
        )

        assertEquals(setOf(gemNode.url, remainingNode.url), visibleStates.keys)
    }

    @Test
    fun `buildNodeRows uses provided default urls for delete eligibility`() {
        val gemNode = getGemNode(Chain.Bitcoin)
        val defaultNode = Node(
            url = "https://default.example.com/bitcoin",
            status = NodeState.Active,
            priority = 0,
        )
        val customNode = Node(
            url = "https://custom.example.com/bitcoin",
            status = NodeState.Active,
            priority = 0,
        )

        val rows = buildNodeRows(
            chain = Chain.Bitcoin,
            nodes = listOf(gemNode, defaultNode, customNode),
            currentNode = gemNode,
            nodeStates = mapOf(customNode.url to NodeStatusState.Error),
            defaultNodeUrls = setOf(defaultNode.url),
        )

        assertFalse(rows.first { it.node.url == gemNode.url }.canDelete)
        assertFalse(rows.first { it.node.url == defaultNode.url }.canDelete)
        assertTrue(rows.first { it.node.url == customNode.url }.canDelete)
        assertEquals(NodeStatusState.Error, rows.first { it.node.url == customNode.url }.statusState)
    }

    @Test
    fun `toStatusState maps zero latest block to error`() {
        val status = NodeStatus(
            url = "https://rpc.example.com/bitcoin",
            chainId = "bitcoin",
            blockNumber = 0UL,
            inSync = true,
            latency = 20UL,
        )

        assertEquals(NodeStatusState.Error, status.toStatusState())
    }

    @Test
    fun `toStatusState keeps successful node responses as result`() {
        val status = NodeStatus(
            url = "https://rpc.example.com/bitcoin",
            chainId = "bitcoin",
            blockNumber = 42UL,
            inSync = true,
            latency = 20UL,
        )

        assertEquals(
            NodeStatusState.Result(
                latestBlock = 42UL,
                latency = 20UL,
                chainId = "bitcoin",
            ),
            status.toStatusState()
        )
    }
}
