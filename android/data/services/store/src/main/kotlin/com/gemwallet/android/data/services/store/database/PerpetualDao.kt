package com.gemwallet.android.data.services.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy.Companion.IGNORE
import androidx.room.Query
import androidx.room.Transaction
import androidx.room.Update
import com.gemwallet.android.data.services.store.database.entities.DbPerpetual
import com.gemwallet.android.data.services.store.database.entities.DbPerpetualData
import com.gemwallet.android.data.services.store.database.entities.DbPerpetualUpdate
import com.gemwallet.android.data.services.store.database.entities.toUpdate
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

    @Transaction
    @Query(
        """
        SELECT * FROM perpetuals
        WHERE :requiresVolume = 0 OR volume24h > 0 OR isPinned
        ORDER BY isPinned DESC, volume24h DESC
        LIMIT :limit
    """,
    )
    fun getPerpetualsData(requiresVolume: Boolean, limit: Int): Flow<List<DbPerpetualData>>

    @Transaction
    @Query(
        """
        SELECT perpetuals.* FROM perpetuals
        JOIN asset ON asset.id = perpetuals.assetId
        WHERE perpetuals.name LIKE '%' || :search || '%' OR asset.symbol LIKE '%' || :search || '%'
        ORDER BY perpetuals.isPinned DESC, perpetuals.volume24h DESC
        LIMIT :limit
    """,
    )
    fun searchPerpetualsData(search: String, limit: Int): Flow<List<DbPerpetualData>>

    @Query("SELECT * FROM perpetuals WHERE name IN (:names)")
    suspend fun getPerpetualsByNames(names: List<String>): List<DbPerpetual>

    @Transaction
    @Query("SELECT * FROM perpetuals WHERE id = :perpetualId")
    fun getPerpetual(perpetualId: String): Flow<DbPerpetualData?>

    @Transaction
    @Query("SELECT * FROM perpetuals WHERE assetId = :assetId LIMIT 1")
    fun getPerpetualByAssetId(assetId: String): Flow<DbPerpetualData?>

    @Query("DELETE FROM perpetuals")
    suspend fun deleteAll()

    @Query("UPDATE perpetuals SET isPinned = :isPinned WHERE id IN (:perpetualIds)")
    suspend fun setPinned(perpetualIds: List<String>, isPinned: Boolean)

    @Query(
        "UPDATE perpetuals SET price = :price, pricePercentChange24h = :pricePercentChange24h, " +
            "openInterest = :openInterest, volume24h = :volume24h, funding = :funding WHERE name = :coin",
    )
    suspend fun updateMarket(coin: String, price: Double, pricePercentChange24h: Double, openInterest: Double, volume24h: Double, funding: Double)

    @Query("UPDATE perpetuals SET price = :price WHERE name = :name")
    suspend fun updatePrice(name: String, price: Double)

    @Transaction
    suspend fun updatePrices(prices: Map<String, Double>) {
        prices.forEach { (name, price) -> updatePrice(name, price) }
    }
}
