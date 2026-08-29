package com.gemwallet.android.data.coordinators.notifications

import com.gemwallet.android.application.notifications.cases.GetInAppNotifications
import com.gemwallet.android.data.repositories.gemstone.GemstoneNotificationStore
import com.wallet.core.primitives.InAppNotification
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class GetInAppNotificationsImpl(
    private val notificationStore: GemstoneNotificationStore,
) : GetInAppNotifications {

    override fun invoke(walletId: WalletId): Flow<List<InAppNotification>> =
        notificationStore.observeNotifications(walletId)
}
