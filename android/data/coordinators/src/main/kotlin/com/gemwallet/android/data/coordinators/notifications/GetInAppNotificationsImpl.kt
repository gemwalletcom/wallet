package com.gemwallet.android.data.coordinators.notifications

import com.gemwallet.android.application.notifications.cases.GetInAppNotifications
import com.gemwallet.android.data.service.store.database.InAppNotificationsDao
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.wallet.core.primitives.InAppNotification
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class GetInAppNotificationsImpl(
    private val notificationsDao: InAppNotificationsDao,
) : GetInAppNotifications {

    override fun invoke(walletId: WalletId): Flow<List<InAppNotification>> =
        notificationsDao.getNotifications(walletId.id).map { records -> records.map { it.toModel() } }
}
