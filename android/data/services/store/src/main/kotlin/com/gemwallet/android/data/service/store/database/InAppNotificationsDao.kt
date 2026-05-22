package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import com.gemwallet.android.data.service.store.database.entities.DbInAppNotification
import kotlinx.coroutines.flow.Flow

@Dao
interface InAppNotificationsDao {

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun put(notifications: List<DbInAppNotification>)

    @Query("SELECT * FROM in_app_notifications WHERE wallet_id = :walletId ORDER BY created_at DESC")
    fun getNotifications(walletId: String): Flow<List<DbInAppNotification>>
}
