package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy.Companion.IGNORE
import androidx.room.Query
import androidx.room.Transaction
import androidx.room.Update
import com.gemwallet.android.data.service.store.database.entities.DbPerpetual
import com.gemwallet.android.data.service.store.database.entities.DbPerpetualData
import com.gemwallet.android.data.service.store.database.entities.DbPerpetualUpdate
import com.gemwallet.android.data.service.store.database.entities.toUpdate
import kotlinx.coroutines.flow.Flow

@Dao
interface PerpetualDao {

    @Insert(onConflict = IGNORE)
    suspend fun insert(items: List<DbPerpetual>)

    @Update(entity = DbPerpetual::class)
    suspend fun update(items: List<DbPerpetualUpdate>)

    @Transaction
    suspend fun upsert(items: List<DbPerpetual>) {
        insert(items)
        update(items.map(DbPerpetual::toUpdate))
    }

    @Query("SELECT * FROM perpetuals")
    fun getPerpetuals(): Flow<List<DbPerpetual>>

    @Transaction
    @Query("SELECT * FROM perpetuals WHERE volume24h > 0 ORDER BY volume24h DESC")
    fun getPerpetualsData(): Flow<List<DbPerpetualData>>

    @Transaction
    @Query("""
        SELECT perpetuals.* FROM perpetuals
        JOIN search ON perpetuals.id = search.perpetualId
        WHERE search.`query` = :query
        ORDER BY search.priority ASC, perpetuals.volume24h DESC
    """)
    fun searchWithPriority(query: String): Flow<List<DbPerpetualData>>

    @Transaction
    @Query("SELECT * FROM perpetuals WHERE id = :perpetualId")
    fun getPerpetual(perpetualId: String): Flow<DbPerpetualData?>

    @Transaction
    @Query("SELECT * FROM perpetuals WHERE assetId = :assetId LIMIT 1")
    fun getPerpetualByAssetId(assetId: String): Flow<DbPerpetualData?>

    @Query("UPDATE perpetuals SET isPinned = :isPinned WHERE id = :perpetualId")
    suspend fun setPinned(perpetualId: String, isPinned: Boolean)

    @Query(
        "UPDATE perpetuals SET price = :price, pricePercentChange24h = :pricePercentChange24h, " +
            "openInterest = :openInterest, volume24h = :volume24h, funding = :funding WHERE name = :coin"
    )
    suspend fun updateMarket(
        coin: String,
        price: Double,
        pricePercentChange24h: Double,
        openInterest: Double,
        volume24h: Double,
        funding: Double,
    )

    @Query("UPDATE perpetuals SET price = :price WHERE name = :name")
    suspend fun updatePrice(name: String, price: Double)

    @Transaction
    suspend fun updatePrices(prices: Map<String, Double>) {
        prices.forEach { (name, price) -> updatePrice(name, price) }
    }
}
