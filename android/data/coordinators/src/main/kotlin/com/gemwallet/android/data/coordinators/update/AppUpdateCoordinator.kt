package com.gemwallet.android.data.coordinators.update

import com.gemwallet.android.application.update.cases.ObserveAppUpdateOffer
import com.gemwallet.android.application.update.cases.SkipAppUpdate
import com.gemwallet.android.application.update.cases.SyncAppUpdate
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.BuildInfo
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAppUpdateOffer
import uniffi.gemstone.GemAppUpdateService
import uniffi.gemstone.GemAppUpdateServiceInterface

class AppUpdateCoordinator(private val appUpdateService: GemAppUpdateServiceInterface, private val buildInfo: BuildInfo) :
    SyncAppUpdate,
    ObserveAppUpdateOffer,
    SkipAppUpdate {

    private val offer = MutableStateFlow<GemAppUpdateOffer?>(null)

    override suspend fun syncAppUpdate(): GemAppUpdateOffer? = check().also { offer.value = it }

    override fun observeAppUpdateOffer(): Flow<GemAppUpdateOffer?> = offer

    override suspend fun skipAppUpdate(update: GemAppUpdateOffer) {
        withContext(Dispatchers.IO) { appUpdateService.skip(update) }
        offer.value = check()
    }

    private suspend fun check(): GemAppUpdateOffer? = withContext(Dispatchers.IO) {
        runCatching { appUpdateService.check(buildInfo.platformStore.toGem(), buildInfo.versionName) }.getOrNull()
    }
}
