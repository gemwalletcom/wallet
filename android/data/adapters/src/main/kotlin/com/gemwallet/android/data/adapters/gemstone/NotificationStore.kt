package com.gemwallet.android.data.adapters.gemstone

import com.gemwallet.android.data.service.store.database.InAppNotificationsDao
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.wallet.core.primitives.InAppNotification
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemNotificationStore

class GemstoneNotificationStore(
    private val notificationsDao: InAppNotificationsDao,
) : GemNotificationStore {

    override suspend fun saveNotifications(notifications: List<uniffi.gemstone.InAppNotification>) =
        notificationsDao.put(notifications.map { it.decodeJson<InAppNotification>().toRecord() })

    fun observeNotifications(walletId: WalletId): Flow<List<InAppNotification>> =
        notificationsDao.getNotifications(walletId.id).map { records -> records.map { it.toModel() } }
}
