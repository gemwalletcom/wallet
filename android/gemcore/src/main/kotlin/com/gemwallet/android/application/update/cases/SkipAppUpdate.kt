package com.gemwallet.android.application.update.cases

import com.gemwallet.android.model.AppUpdateOffer

interface SkipAppUpdate {
    suspend fun skipAppUpdate(update: AppUpdateOffer)
}
