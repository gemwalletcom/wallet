package com.gemwallet.android.cases.nodes

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node
import com.wallet.core.primitives.NodeState
import uniffi.gemstone.Config
import uniffi.gemstone.NodeRegion

private val config = Config()

private fun NodeRegion.toNode(chain: Chain) = Node(
    url = getGemNodeUrl(chain, this),
    status = NodeState.Active,
    priority = config.getNodeRegionPriority(this),
)

fun getGemNodeUrl(chain: Chain, region: NodeRegion = NodeRegion.US) = config.getNodeUrl(chain.string, region)

fun getGemNode(chain: Chain, region: NodeRegion = NodeRegion.US) = region.toNode(chain)

fun getGemNodes(chain: Chain): List<Node> = config.getNodeRegions().map { getGemNode(chain, it) }

fun getGemNodeUrls(chain: Chain): Set<String> = getGemNodes(chain).mapTo(linkedSetOf(), Node::url)
