package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy.Companion.REPLACE
import androidx.room.Query
import androidx.room.Transaction
import com.gemwallet.android.data.service.store.database.entities.DbPerpetualPosition
import com.gemwallet.android.data.service.store.database.entities.DbPerpetualPositionData
import kotlinx.coroutines.flow.Flow

@Dao
interface PerpetualPositionDao {

    @Insert(onConflict = REPLACE)
    suspend fun upsertPositions(items: List<DbPerpetualPosition>)

    @Query("DELETE FROM perpetuals_positions WHERE walletId = :walletId AND id NOT IN (:ids)")
    suspend fun deleteStale(walletId: String, ids: List<String>)

    @Transaction
    suspend fun diffPositions(walletId: String, items: List<DbPerpetualPosition>) {
        deleteStale(walletId, items.map { it.id })
        upsertPositions(items)
    }

    @Transaction
    @Query("SELECT * FROM perpetuals_positions WHERE walletId = :walletId ORDER BY updatedAt DESC")
    fun getPositionsData(walletId: String): Flow<List<DbPerpetualPositionData>>

    @Transaction
    @Query("SELECT * FROM perpetuals_positions WHERE id = :positionId")
    fun getPositionData(positionId: String): Flow<DbPerpetualPositionData?>

    @Transaction
    @Query("SELECT * FROM perpetuals_positions WHERE perpetualId = :perpetualId LIMIT 1")
    fun getPositionDataByPerpetual(perpetualId: String): Flow<DbPerpetualPositionData?>
}
