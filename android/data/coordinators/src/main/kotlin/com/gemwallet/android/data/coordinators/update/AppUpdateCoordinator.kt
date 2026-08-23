package com.gemwallet.android.data.coordinators.update

import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.application.update.coordinators.ObserveAppUpdateOffer
import com.gemwallet.android.application.update.coordinators.SkipAppUpdate
import com.gemwallet.android.application.update.coordinators.SyncAppUpdate
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.ext.VersionCheck
import com.gemwallet.android.model.AppUpdateChannel
import com.gemwallet.android.model.AppUpdateInfo
import com.gemwallet.android.model.AppUpdateOffer
import com.gemwallet.android.model.BuildInfo
import com.wallet.core.primitives.PlatformStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.firstOrNull

class AppUpdateCoordinator(
    private val getRemoteConfig: GetRemoteConfig,
    private val userConfig: UserConfig,
    private val buildInfo: BuildInfo,
) : SyncAppUpdate, ObserveAppUpdateOffer, SkipAppUpdate {

    override suspend fun syncAppUpdate(): AppUpdateOffer? {
        if (buildInfo.platformStore == PlatformStore.Local) {
            return null
        }
        val config = runCatching { getRemoteConfig.getRemoteConfig() }.getOrNull() ?: return null
        val release = config.releases.firstOrNull { it.store == buildInfo.platformStore } ?: return null
        val skippedVersion = userConfig.getAppVersionSkip().firstOrNull().orEmpty()
        val update = AppUpdateInfo(version = release.version, isRequired = release.upgradeRequired)
        userConfig.setLatestAppUpdate(update)
        return buildOffer(update, skippedVersion)
    }

    override fun observeAppUpdateOffer(): Flow<AppUpdateOffer?> = combine(
        userConfig.getLatestAppUpdate(),
        userConfig.getAppVersionSkip(),
    ) { update, skippedVersion -> buildOffer(update, skippedVersion) }

    override suspend fun skipAppUpdate(version: String) {
        userConfig.setAppVersionSkip(version)
    }

    private fun buildOffer(update: AppUpdateInfo?, skippedVersion: String): AppUpdateOffer? {
        if (update == null) {
            return null
        }
        if (!VersionCheck.isVersionHigher(new = update.version, current = buildInfo.versionName)) {
            return null
        }
        if (!update.isRequired && update.version == skippedVersion) {
            return null
        }
        return AppUpdateOffer(
            version = update.version,
            isRequired = update.isRequired,
            channel = deliveryChannel(),
        )
    }

    private fun deliveryChannel(): AppUpdateChannel = when (buildInfo.platformStore) {
        PlatformStore.ApkUniversal -> AppUpdateChannel.InAppApk
        else -> AppUpdateChannel.Store
    }
}
