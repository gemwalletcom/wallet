package com.gemwallet.android.data.coordinators.update

import com.gemwallet.android.application.update.cases.ObserveAppUpdateOffer
import com.gemwallet.android.application.update.cases.SkipAppUpdate
import com.gemwallet.android.application.update.cases.SyncAppUpdate
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AppUpdateChannel
import com.gemwallet.android.model.AppUpdateOffer
import com.gemwallet.android.model.BuildInfo
import com.wallet.core.primitives.PlatformStore
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

    private val offer = MutableStateFlow<AppUpdateOffer?>(null)

    override suspend fun syncAppUpdate(): AppUpdateOffer? = check().also { offer.value = it }

    override fun observeAppUpdateOffer(): Flow<AppUpdateOffer?> = offer

    override suspend fun skipAppUpdate(update: AppUpdateOffer) {
        withContext(Dispatchers.IO) { appUpdateService.skip(GemAppUpdateOffer(update.version, update.canSkip)) }
        offer.value = check()
    }

    private suspend fun check(): AppUpdateOffer? {
        val update = withContext(Dispatchers.IO) {
            runCatching { appUpdateService.check(buildInfo.platformStore.toGem(), buildInfo.versionName) }.getOrNull()
        } ?: return null
        return AppUpdateOffer(
            version = update.version,
            canSkip = update.canSkip,
            channel = deliveryChannel(),
        )
    }

    private fun deliveryChannel(): AppUpdateChannel = when (buildInfo.platformStore) {
        PlatformStore.ApkUniversal -> AppUpdateChannel.InAppApk
        else -> AppUpdateChannel.Store
    }
}
