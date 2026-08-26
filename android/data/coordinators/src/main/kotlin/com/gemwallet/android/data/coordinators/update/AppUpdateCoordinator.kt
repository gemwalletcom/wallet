package com.gemwallet.android.data.coordinators.update

import com.gemwallet.android.application.update.coordinators.ObserveAppUpdateOffer
import com.gemwallet.android.application.update.coordinators.SkipAppUpdate
import com.gemwallet.android.application.update.coordinators.SyncAppUpdate
import com.gemwallet.android.model.AppUpdateChannel
import com.gemwallet.android.model.AppUpdateOffer
import com.gemwallet.android.model.BuildInfo
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.PlatformStore
import com.wallet.core.primitives.Release
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import uniffi.gemstone.GemAppUpdateService

class AppUpdateCoordinator(
    private val appUpdateService: GemAppUpdateService,
    private val buildInfo: BuildInfo,
) : SyncAppUpdate, ObserveAppUpdateOffer, SkipAppUpdate {

    private val offer = MutableStateFlow<AppUpdateOffer?>(null)

    override suspend fun syncAppUpdate(): AppUpdateOffer? = check().also { offer.value = it }

    override fun observeAppUpdateOffer(): Flow<AppUpdateOffer?> = offer

    override suspend fun skipAppUpdate(version: String) {
        appUpdateService.skip(version)
        offer.value = check()
    }

    private suspend fun check(): AppUpdateOffer? {
        val release = runCatching { appUpdateService.check(buildInfo.platformStore.toJson(), buildInfo.versionName) }
            .getOrNull()?.decodeJson<Release>() ?: return null
        return AppUpdateOffer(
            version = release.version,
            isRequired = release.upgradeRequired,
            channel = deliveryChannel(),
        )
    }

    private fun deliveryChannel(): AppUpdateChannel = when (buildInfo.platformStore) {
        PlatformStore.ApkUniversal -> AppUpdateChannel.InAppApk
        else -> AppUpdateChannel.Store
    }
}
