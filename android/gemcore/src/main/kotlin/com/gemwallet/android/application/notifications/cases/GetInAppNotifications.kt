package com.gemwallet.android.application.notifications.cases

import com.wallet.core.primitives.InAppNotification
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface GetInAppNotifications {
    operator fun invoke(walletId: WalletId): Flow<List<InAppNotification>>
}
