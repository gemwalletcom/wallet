package com.gemwallet.android.application.update.cases

import com.gemwallet.android.model.AppUpdateOffer

interface SyncAppUpdate {
    suspend fun syncAppUpdate(): AppUpdateOffer?
}
