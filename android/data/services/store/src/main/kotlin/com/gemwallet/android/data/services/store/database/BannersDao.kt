package com.gemwallet.android.data.services.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Transaction
import com.gemwallet.android.data.services.store.database.entities.DbBanner
import com.gemwallet.android.data.services.store.database.entities.DbBannerWithAsset
import com.wallet.core.primitives.BannerState
import kotlinx.coroutines.flow.Flow

@Dao
interface BannersDao {
    @Query("SELECT * FROM banners WHERE id = :id")
    suspend fun getBanner(id: String): DbBanner?

    @Transaction
    @Query("SELECT * FROM banners")
    fun observeBanners(): Flow<List<DbBannerWithAsset>>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun saveBanner(banner: DbBanner)

    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun addBanners(banners: List<DbBanner>)

    @Query("DELETE FROM banners")
    suspend fun deleteAll()

    @Query("UPDATE banners SET state = :to WHERE state = :from")
    suspend fun updateStates(from: BannerState, to: BannerState)
}
