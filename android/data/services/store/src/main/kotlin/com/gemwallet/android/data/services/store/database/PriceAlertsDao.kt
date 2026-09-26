package com.gemwallet.android.data.services.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Transaction
import com.gemwallet.android.data.services.store.database.entities.DbPriceAlert
import com.gemwallet.android.data.services.store.database.entities.DbPriceAlertWithAsset
import kotlinx.coroutines.flow.Flow

@Dao
interface PriceAlertsDao {

    @Query("DELETE FROM price_alerts")
    suspend fun clear()

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun put(alerts: List<DbPriceAlert>)

    @Query("SELECT * FROM price_alerts")
    fun getAlerts(): Flow<List<DbPriceAlert>>

    @Query("SELECT * FROM price_alerts WHERE assetId = :assetId")
    fun getAlerts(assetId: String): Flow<List<DbPriceAlert>>

    @Transaction
    @Query("SELECT price_alerts.* FROM price_alerts JOIN asset ON asset.id = price_alerts.assetId ORDER BY asset.rank DESC")
    fun getAlertsWithAsset(): Flow<List<DbPriceAlertWithAsset>>

    @Transaction
    @Query("SELECT price_alerts.* FROM price_alerts JOIN asset ON asset.id = price_alerts.assetId WHERE price_alerts.assetId = :assetId ORDER BY asset.rank DESC")
    fun getAlertsWithAsset(assetId: String): Flow<List<DbPriceAlertWithAsset>>

    @Query("SELECT * FROM price_alerts")
    suspend fun getAllPriceAlerts(): List<DbPriceAlert>

    @Query("SELECT * FROM price_alerts WHERE assetId = :assetId")
    suspend fun getAllPriceAlerts(assetId: String): List<DbPriceAlert>

    @Query("DELETE FROM price_alerts WHERE id IN (:ids)")
    suspend fun delete(ids: List<String>)

    @Transaction
    suspend fun update(alerts: List<DbPriceAlert>, deleteIds: List<String>) {
        if (deleteIds.isNotEmpty()) {
            delete(deleteIds)
        }
        if (alerts.isNotEmpty()) {
            put(alerts)
        }
    }
}
