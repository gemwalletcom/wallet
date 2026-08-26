package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.data.service.store.database.InAppNotificationsDao
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.InAppNotification
import uniffi.gemstone.GemNotificationStore

class GemstoneNotificationStore(
    private val notificationsDao: InAppNotificationsDao,
    private val walletPreferencesFactory: WalletPreferencesFactory,
) : GemNotificationStore {

    override suspend fun save(notifications: List<uniffi.gemstone.InAppNotification>) =
        notificationsDao.put(notifications.map { it.decodeJson<InAppNotification>().toRecord() })

    override suspend fun getSyncTimestamp(walletId: String): ULong =
        walletPreferencesFactory.create(walletId).notificationsTimestamp.toULong()

    override suspend fun setSyncTimestamp(walletId: String, timestamp: ULong) {
        walletPreferencesFactory.create(walletId).notificationsTimestamp = timestamp.toLong()
    }
}
