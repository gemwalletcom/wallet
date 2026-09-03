package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import com.gemwallet.android.data.service.store.database.entities.DbBanner
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.BannerState
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.Flow

@Dao
interface BannersDao {
    @Query("SELECT * FROM banners WHERE id = :id")
    suspend fun getBanner(id: String): DbBanner?

    @Query("SELECT * FROM banners WHERE id = :id")
    fun observeBanner(id: String): Flow<DbBanner?>

    @Query("""
        SELECT * FROM
            banners
        WHERE
            (wallet_id IS NULL OR wallet_id = :walletId)
            AND asset_id = :assetId
    """)
    fun observeAssetBanners(walletId: String?, assetId: String): Flow<List<DbBanner>>

    @Query("""
        SELECT * FROM
            banners
        WHERE
            wallet_id = :walletId AND event IN (:events)
    """)
    fun observeWalletBanners(walletId: String, events: List<BannerEvent>): Flow<List<DbBanner>>

    @Query("""
        SELECT state FROM
            banners
        WHERE
            wallet_id=:walletId AND event = "AccountBlockedMultiSignature"
    """)
    fun getMultisign(walletId: String): Flow<List<BannerState>>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun saveBanner(banner: DbBanner)

    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun addBanners(banners: List<DbBanner>)
}
