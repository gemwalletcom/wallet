package com.gemwallet.android.data.service.store.database

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.toGem
import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import com.gemwallet.android.data.service.store.database.entities.DbNode
import com.gemwallet.android.ext.requireChain
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node
import uniffi.gemstone.GemNodeStore

@Dao
interface NodesDao : GemNodeStore {

    override suspend fun getNodes(chain: String): List<uniffi.gemstone.Node> = getNodes(chain.requireChain())
        .map { Node(it.url, it.status, it.priority).toGem() }

    override suspend fun addNode(chain: String, node: uniffi.gemstone.Node) {
        val value = node.toPrimitives()
        addNodes(listOf(DbNode(value.url, value.status, value.priority, chain.requireChain())))
    }

    override suspend fun deleteNode(chain: String, url: String) = deleteNode(chain.requireChain(), url)

    @Insert(onConflict = OnConflictStrategy.Companion.REPLACE)
    suspend fun addNodes(nodes: List<DbNode>)

    @Query("DELETE FROM nodes WHERE chain = :chain AND url = :url")
    suspend fun deleteNode(chain: Chain, url: String)

    @Query("SELECT * FROM nodes WHERE chain = :chain ORDER BY priority DESC, url ASC")
    suspend fun getNodes(chain: Chain): List<DbNode>

}
