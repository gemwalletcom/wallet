package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.InAppNotificationsDao
import com.gemwallet.android.data.services.store.database.entities.toModel
import com.wallet.core.primitives.InAppNotification
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class InAppNotificationsQuery @Inject constructor(private val notificationsDao: InAppNotificationsDao) {

    operator fun invoke(walletId: String): Flow<List<InAppNotification>> = notificationsDao.getNotifications(walletId).map { records -> records.map { it.toModel() } }
}
