package com.gemwallet.android.features.settings.networks.viewmodels

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemNodeSelection
import uniffi.gemstone.GemNodeStatusState
import uniffi.gemstone.Latency
import uniffi.gemstone.LatencyType

class NetworksNodeUiStateTest {

    @Test
    fun `visibleNodeStates removes deleted node entries`() {
        val gemNode = selection("https://gemnodes.com/bitcoin")
        val remainingNode = selection("https://custom.example.com/bitcoin")
        val deletedNode = selection("https://deleted.example.com/bitcoin")
        val nodeStates = mapOf(
            gemNode.url to GemNodeStatusState.Loading,
            remainingNode.url to GemNodeStatusState.Error,
            deletedNode.url to GemNodeStatusState.Result(
                latestBlockNumber = 1UL,
                latency = Latency(LatencyType.FAST, 20.0),
            ),
        )

        val visibleStates = visibleNodeStates(
            nodes = rows(gemNode, remainingNode),
            nodeStates = nodeStates,
        )

        assertEquals(setOf(gemNode.url, remainingNode.url), visibleStates.keys)
    }

    @Test
    fun `buildNodeRows marks rows the delete rule allows`() {
        val gemNode = selection("https://gemnodes.com/bitcoin")
        val defaultNode = selection("https://default.example.com/bitcoin")
        val customNode = selection("https://custom.example.com/bitcoin")

        val rows = buildNodeRows(
            selections = listOf(gemNode, defaultNode, customNode),
            gemNodeFlag = { url -> "🇺🇸".takeIf { url == gemNode.url } },
            canDelete = { url -> url == customNode.url },
        )

        assertFalse(rows.first { it.url == gemNode.url }.canDelete)
        assertFalse(rows.first { it.url == defaultNode.url }.canDelete)
        assertTrue(rows.first { it.url == customNode.url }.canDelete)
    }

    @Test
    fun `buildNodeRows selects the node core marked on a shared host`() {
        val firstNode = selection("https://rpc.example.com/one")
        val secondNode = selection("https://rpc.example.com/two", isSelected = true)

        val rows = buildNodeRows(
            selections = listOf(firstNode, secondNode),
            gemNodeFlag = { null },
            canDelete = { true },
        )

        assertEquals(listOf(secondNode.url), rows.filter { it.selected }.map { it.url })
    }

    private fun selection(url: String, isSelected: Boolean = false) = GemNodeSelection(
        url = url,
        host = url.removePrefix("https://").substringBefore("/"),
        isSelected = isSelected,
    )

    private fun rows(vararg selections: GemNodeSelection) = buildNodeRows(
        selections = selections.toList(),
        gemNodeFlag = { null },
        canDelete = { false },
    )
}
