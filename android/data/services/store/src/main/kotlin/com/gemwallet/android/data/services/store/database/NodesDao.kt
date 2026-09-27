package com.gemwallet.android.data.services.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import com.gemwallet.android.data.services.store.database.entities.DbNode
import com.wallet.core.primitives.Chain

@Dao
interface NodesDao {

    @Insert(onConflict = OnConflictStrategy.Companion.REPLACE)
    suspend fun addNodes(nodes: List<DbNode>)

    @Query("DELETE FROM nodes WHERE chain = :chain AND url = :url")
    suspend fun deleteNode(chain: Chain, url: String)

    @Query("SELECT * FROM nodes WHERE chain = :chain ORDER BY priority DESC, url ASC")
    suspend fun getNodes(chain: Chain): List<DbNode>
}
