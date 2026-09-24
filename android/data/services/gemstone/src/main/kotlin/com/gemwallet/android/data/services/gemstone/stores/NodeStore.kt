package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.NodesDao
import com.gemwallet.android.data.service.store.database.entities.DbNode
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.Node
import uniffi.gemstone.GemNodeStore

class GemstoneNodeStore(private val nodesDao: NodesDao) : GemNodeStore {

    override suspend fun getNodes(chain: String): List<uniffi.gemstone.Node> = nodesDao.getNodes(chain.requireChain())
        .map { Node(it.url, it.status, it.priority).toGem() }

    override suspend fun addNode(chain: String, node: uniffi.gemstone.Node) {
        val value = node.toPrimitives()
        nodesDao.addNodes(listOf(DbNode(value.url, value.status, value.priority, chain.requireChain())))
    }

    override suspend fun deleteNode(chain: String, url: String) = nodesDao.deleteNode(chain.requireChain(), url)
}
