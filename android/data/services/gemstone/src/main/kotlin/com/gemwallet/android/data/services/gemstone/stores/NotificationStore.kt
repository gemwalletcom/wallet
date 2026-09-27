package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.InAppNotificationsDao
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.GemNotificationStore

class GemstoneNotificationStore(private val notificationsDao: InAppNotificationsDao) : GemNotificationStore {

    override suspend fun saveNotifications(notifications: List<uniffi.gemstone.InAppNotification>) = notificationsDao.put(notifications.map { it.toPrimitives().toRecord() })

    override suspend fun markNotificationsRead(walletId: String, createdBefore: Long) = notificationsDao.markRead(walletId, createdBefore, System.currentTimeMillis())

    override suspend fun hasUnreadNotifications(walletId: String): Boolean = notificationsDao.hasUnread(walletId)
}
