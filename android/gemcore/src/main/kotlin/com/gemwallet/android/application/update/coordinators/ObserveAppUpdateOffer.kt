package com.gemwallet.android.application.update.coordinators

import com.gemwallet.android.model.AppUpdateOffer
import kotlinx.coroutines.flow.Flow

interface ObserveAppUpdateOffer {
    fun observeAppUpdateOffer(): Flow<AppUpdateOffer?>
}
