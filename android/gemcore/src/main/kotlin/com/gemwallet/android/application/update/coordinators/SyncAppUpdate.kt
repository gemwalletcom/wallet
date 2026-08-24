package com.gemwallet.android.application.update.coordinators

import com.gemwallet.android.model.AppUpdateOffer

interface SyncAppUpdate {
    suspend fun syncAppUpdate(): AppUpdateOffer?
}
