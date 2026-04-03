package com.gemwallet.android.cases.nodes

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NodeState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class GemNodeCaseTest {

    @Test
    fun `getGemNodes returns regional nodes in ios order`() {
        val nodes = getGemNodes(Chain.Bitcoin)

        assertEquals(
            listOf(
                "https://gemnodes.com/bitcoin",
                "https://asia.gemnodes.com/bitcoin",
                "https://eu.gemnodes.com/bitcoin",
            ),
            nodes.map { it.url }
        )
        assertEquals(listOf(10, 8, 9), nodes.map { it.priority })
        assertEquals(listOf(NodeState.Active, NodeState.Active, NodeState.Active), nodes.map { it.status })
    }

    @Test
    fun `getGemNode defaults to us region`() {
        assertEquals(
            getGemNode(Chain.Solana, GemNodeRegion.US),
            getGemNode(Chain.Solana),
        )
    }
}
