package com.gemwallet.android.data.repositories.notifications

import com.gemwallet.android.data.service.store.database.InAppNotificationsDao
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.wallet.core.primitives.InAppNotification
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemNotificationService

class InAppNotificationsRepository(
    private val notificationService: GemNotificationService,
    private val notificationsDao: InAppNotificationsDao,
) {

    fun getNotifications(walletId: WalletId): Flow<List<InAppNotification>> =
        notificationsDao.getNotifications(walletId.id).map { records -> records.map { it.toModel() } }

    suspend fun sync(walletId: WalletId) = notificationService.sync(walletId.id)

    suspend fun addNotification(notification: InAppNotification) {
        notificationsDao.put(listOf(notification.toRecord()))
    }

    suspend fun markNotificationsRead() = notificationService.markRead()
}
