package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import com.gemwallet.android.data.service.store.database.entities.DbFiatRate
import com.gemwallet.android.data.service.store.database.entities.DbPrice
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.flow.Flow

@Dao
interface PricesDao {

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(priceRoom: DbPrice)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(priceRoom: List<DbPrice>)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun setRates(rates: List<DbFiatRate>)

    @Query("UPDATE prices SET value = usd_value * :rate, currency = :currency")
    suspend fun updateValues(currency: Currency, rate: Double)

    @Query("SELECT * FROM prices")
    fun getAll(): Flow<List<DbPrice>>

    @Query("SELECT * FROM prices WHERE asset_id IN (:assetsId)")
    fun getByAssets(assetsId: List<String>): List<DbPrice>

    @Query("SELECT usd_value FROM prices WHERE asset_id = :assetId LIMIT 1")
    fun getUsdPrice(assetId: String): Flow<Double?>

    @Query("DELETE FROM prices")
    suspend fun deleteAll()

    @Query("SELECT * FROM currency_rates WHERE currency=:currency")
    fun getRates(currency: Currency): Flow<DbFiatRate?>
}
