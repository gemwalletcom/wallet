package com.gemwallet.android.cases.nodes

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node
import com.wallet.core.primitives.NodeState

enum class GemNodeRegion(
    val baseUrl: String,
    val flag: String,
    private val nodePriority: Int,
) {
    US(
        baseUrl = "https://gemnodes.com",
        flag = "\uD83C\uDDFA\uD83C\uDDF8",
        nodePriority = 10,
    ),
    ASIA(
        baseUrl = "https://asia.gemnodes.com",
        flag = "\uD83C\uDDEF\uD83C\uDDF5",
        nodePriority = 8,
    ),
    EU(
        baseUrl = "https://eu.gemnodes.com",
        flag = "\uD83C\uDDEA\uD83C\uDDFA",
        nodePriority = 9,
    ),
    ;

    fun toNode(chain: Chain): Node = Node(
        url = getGemNodeUrl(chain, this),
        status = NodeState.Active,
        priority = nodePriority,
    )
}

fun getGemNodeUrl(chain: Chain, region: GemNodeRegion = GemNodeRegion.US) = "${region.baseUrl}/${chain.string}"

fun getGemNode(chain: Chain, region: GemNodeRegion = GemNodeRegion.US) = region.toNode(chain)

fun getGemNodes(chain: Chain): List<Node> = listOf(
    getGemNode(chain, GemNodeRegion.US),
    getGemNode(chain, GemNodeRegion.ASIA),
    getGemNode(chain, GemNodeRegion.EU),
)

fun getGemNodeUrls(chain: Chain): Set<String> = getGemNodes(chain).mapTo(linkedSetOf(), Node::url)

fun getGemNodeRegion(url: String): GemNodeRegion? = GemNodeRegion.entries.firstOrNull { url.startsWith(it.baseUrl) }
