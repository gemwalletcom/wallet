package com.gemwallet.android.features.settings.networks.viewmodels

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node
import com.wallet.core.primitives.NodeState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemNodeStatusState
import uniffi.gemstone.Latency
import uniffi.gemstone.LatencyType

class NetworksNodeUiStateTest {

    @Test
    fun `visibleNodeStates removes deleted node entries`() {
        val gemNode = Node(
            url = "https://gemnodes.com/bitcoin",
            status = NodeState.Active,
            priority = 10,
        )
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
            gemNode.url to GemNodeStatusState.Loading,
            remainingNode.url to GemNodeStatusState.Error,
            deletedNode.url to GemNodeStatusState.Result(
                latestBlockNumber = 1UL,
                latency = Latency(LatencyType.FAST, 20.0),
            ),
        )

        val visibleStates = visibleNodeStates(
            nodes = listOf(gemNode, remainingNode),
            nodeStates = nodeStates,
        )

        assertEquals(setOf(gemNode.url, remainingNode.url), visibleStates.keys)
    }

    @Test
    fun `buildNodeRows marks rows the delete rule allows`() {
        val gemNode = Node(
            url = "https://gemnodes.com/bitcoin",
            status = NodeState.Active,
            priority = 10,
        )
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
            nodes = listOf(gemNode, defaultNode, customNode),
            currentNode = gemNode,
            nodeStates = mapOf(customNode.url to GemNodeStatusState.Error),
            gemNodeFlag = { url -> "🇺🇸".takeIf { url == gemNode.url } },
            canDelete = { url -> url == customNode.url },
        )

        assertFalse(rows.first { it.node.url == gemNode.url }.canDelete)
        assertFalse(rows.first { it.node.url == defaultNode.url }.canDelete)
        assertTrue(rows.first { it.node.url == customNode.url }.canDelete)
        assertEquals(GemNodeStatusState.Error, rows.first { it.node.url == customNode.url }.statusState)
    }
}
