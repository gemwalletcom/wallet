package com.gemwallet.android.data.repositories.notifications

import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.data.service.store.database.InAppNotificationsDao
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.ext.currentTimestamp
import com.wallet.core.primitives.InAppNotification
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class InAppNotificationsRepository(
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val notificationsDao: InAppNotificationsDao,
    private val walletPreferencesFactory: WalletPreferencesFactory,
) {

    fun getNotifications(walletId: WalletId): Flow<List<InAppNotification>> =
        notificationsDao.getNotifications(walletId.id).map { records -> records.map { it.toModel() } }

    suspend fun sync(walletId: WalletId) {
        val preferences = walletPreferencesFactory.create(walletId.id)
        val newTimestamp = currentTimestamp()
        val notifications = gemDeviceApiClient.getNotifications(preferences.notificationsTimestamp)
        notificationsDao.put(notifications.map { it.toRecord() })
        preferences.notificationsTimestamp = newTimestamp
    }

    suspend fun markNotificationsRead() {
        gemDeviceApiClient.markNotificationsRead()
    }
}
