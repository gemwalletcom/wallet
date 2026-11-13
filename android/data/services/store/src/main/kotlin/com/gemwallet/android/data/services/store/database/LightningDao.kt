package com.gemwallet.android.data.services.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.Query
import com.gemwallet.android.data.services.store.database.entities.DbLightningPendingSend

@Dao
interface LightningDao {

    @Query("SELECT * FROM lightning_pending_sends WHERE wallet_id = :walletId ORDER BY transfer_id")
    suspend fun getPendingSends(walletId: String): List<DbLightningPendingSend>

    @Insert
    suspend fun addPendingSend(send: DbLightningPendingSend)

    @Query("DELETE FROM lightning_pending_sends WHERE wallet_id = :walletId AND transfer_id = :transferId")
    suspend fun deletePendingSend(walletId: String, transferId: String)
}
