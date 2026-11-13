package com.gemwallet.android.data.services.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Transaction
import com.gemwallet.android.data.services.store.database.entities.DbLightningLeaf
import com.gemwallet.android.data.services.store.database.entities.DbLightningPendingSend

@Dao
interface LightningDao {

    @Query("SELECT * FROM lightning_leaves WHERE wallet_id = :walletId")
    suspend fun getLeaves(walletId: String): List<DbLightningLeaf>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun putLeaves(leaves: List<DbLightningLeaf>)

    @Query("DELETE FROM lightning_leaves WHERE wallet_id = :walletId AND id IN (:ids)")
    suspend fun deleteLeaves(walletId: String, ids: List<String>)

    @Transaction
    suspend fun updateLeaves(walletId: String, leaves: List<DbLightningLeaf>, deleteIds: List<String>) {
        deleteIds.chunked(DELETE_CHUNK).forEach { deleteLeaves(walletId, it) }
        if (leaves.isNotEmpty()) {
            putLeaves(leaves)
        }
    }

    @Query("SELECT * FROM lightning_pending_sends WHERE wallet_id = :walletId ORDER BY transfer_id")
    suspend fun getPendingSends(walletId: String): List<DbLightningPendingSend>

    @Insert(onConflict = OnConflictStrategy.ABORT)
    suspend fun addPendingSend(send: DbLightningPendingSend)

    @Query("DELETE FROM lightning_pending_sends WHERE wallet_id = :walletId AND transfer_id = :transferId")
    suspend fun deletePendingSend(walletId: String, transferId: String)

    private companion object {
        const val DELETE_CHUNK = 500
    }
}
