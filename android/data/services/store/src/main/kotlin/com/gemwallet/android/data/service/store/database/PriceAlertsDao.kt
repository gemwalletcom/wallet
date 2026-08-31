package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Transaction
import com.gemwallet.android.data.service.store.database.entities.DbPriceAlert
import kotlinx.coroutines.flow.Flow

@Dao
interface PriceAlertsDao {

    @Query("DELETE FROM price_alerts")
    suspend fun clear()

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun put(alerts: List<DbPriceAlert>)

    @Query("SELECT EXISTS(SELECT 1 FROM price_alerts WHERE assetId = :assetId)")
    suspend fun hasAssetPriceAlerts(assetId: String): Boolean

    @Query("SELECT * FROM price_alerts")
    fun getAlerts(): Flow<List<DbPriceAlert>>

    @Query("SELECT * FROM price_alerts WHERE assetId = :assetId")
    fun getAlerts(assetId: String): Flow<List<DbPriceAlert>>

    @Query("SELECT * FROM price_alerts")
    suspend fun getAllPriceAlerts(): List<DbPriceAlert>

    @Query("SELECT * FROM price_alerts WHERE assetId = :assetId")
    suspend fun getAllPriceAlerts(assetId: String): List<DbPriceAlert>

    @Query("SELECT * FROM price_alerts WHERE assetId = :assetId AND price IS NULL AND pricePercentChange IS NULL AND priceDirection IS NULL")
    fun getAssetPriceAlert(assetId: String): Flow<DbPriceAlert?>

    @Query("DELETE FROM price_alerts WHERE id IN (:ids)")
    suspend fun delete(ids: List<String>)

    @Query("SELECT * FROM price_alerts WHERE id = :priceAlertId")
    fun getPriceAlert(priceAlertId: String): DbPriceAlert?

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
