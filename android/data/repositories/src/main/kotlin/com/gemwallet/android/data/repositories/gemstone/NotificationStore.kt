package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.InAppNotificationsDao
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.InAppNotification
import uniffi.gemstone.GemNotificationStore

class GemstoneNotificationStore(
    private val notificationsDao: InAppNotificationsDao,
) : GemNotificationStore {

    override suspend fun saveNotifications(notifications: List<uniffi.gemstone.InAppNotification>) =
        notificationsDao.put(notifications.map { it.decodeJson<InAppNotification>().toRecord() })
}
