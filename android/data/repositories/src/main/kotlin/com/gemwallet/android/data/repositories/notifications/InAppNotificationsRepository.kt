package com.gemwallet.android.data.repositories.notifications

import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.data.service.store.database.InAppNotificationsDao
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.currentTimestamp
import com.wallet.core.primitives.InAppNotification
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemNotificationService
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson

class InAppNotificationsRepository(
    private val notificationService: GemNotificationService,
    private val notificationsDao: InAppNotificationsDao,
    private val walletPreferencesFactory: WalletPreferencesFactory,
) {

    fun getNotifications(walletId: WalletId): Flow<List<InAppNotification>> =
        notificationsDao.getNotifications(walletId.id).map { records -> records.map { it.toModel() } }

    suspend fun sync(walletId: WalletId) {
        val preferences = walletPreferencesFactory.create(walletId.id)
        val newTimestamp = currentTimestamp()
        val notifications = notificationService.getNotifications(preferences.notificationsTimestamp.toULong()).map { it.decodeJson<InAppNotification>() }
        notificationsDao.put(notifications.map { it.toRecord() })
        preferences.notificationsTimestamp = newTimestamp
    }

    suspend fun addNotification(notification: InAppNotification) {
        notificationsDao.put(listOf(notification.toRecord()))
    }

    suspend fun markNotificationsRead() {
        notificationService.markRead()
    }
}
