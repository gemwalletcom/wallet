package com.gemwallet.android.data.coordinators.update

import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.cases.device.RequestPushToken
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.model.AppUpdateChannel
import com.gemwallet.android.model.AppUpdateInfo
import com.gemwallet.android.model.BuildInfo
import com.wallet.core.primitives.ConfigResponse
import com.wallet.core.primitives.ConfigVersions
import com.wallet.core.primitives.PlatformStore
import com.wallet.core.primitives.Release
import com.wallet.core.primitives.SwapConfig
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class AppUpdateCoordinatorTest {

    private val skippedVersion = MutableStateFlow("")
    private val latestAppUpdate = MutableStateFlow<AppUpdateInfo?>(null)
    private val userConfig = mockk<UserConfig>(relaxed = true)
    private val getRemoteConfig = mockk<GetRemoteConfig>()
    private val requestPushToken = mockk<RequestPushToken>(relaxed = true)

    @Before
    fun setUp() {
        every { userConfig.getAppVersionSkip() } returns skippedVersion
        every { userConfig.getLatestAppUpdate() } returns latestAppUpdate
    }

    @Test
    fun `sync persists the release of the current store and offers it`() = runTest {
        remoteReleases(
            Release(version = "2.0.0", store = PlatformStore.Huawei, upgradeRequired = false),
            Release(version = "3.0.0", store = PlatformStore.GooglePlay, upgradeRequired = false),
        )

        val offer = coordinator(PlatformStore.GooglePlay).syncAppUpdate()

        assertEquals("3.0.0", offer?.version)
        assertEquals(AppUpdateChannel.Store, offer?.channel)
        coVerify { userConfig.setLatestAppUpdate(AppUpdateInfo(version = "3.0.0", isRequired = false)) }
    }

    @Test
    fun `sync offers the universal apk build through the in app channel`() = runTest {
        remoteReleases(Release(version = "3.0.0", store = PlatformStore.ApkUniversal, upgradeRequired = false))

        val offer = coordinator(PlatformStore.ApkUniversal).syncAppUpdate()

        assertEquals(AppUpdateChannel.InAppApk, offer?.channel)
    }

    @Test
    fun `sync ignores releases published for another store`() = runTest {
        remoteReleases(Release(version = "3.0.0", store = PlatformStore.Fdroid, upgradeRequired = false))

        val offer = coordinator(PlatformStore.GooglePlay).syncAppUpdate()

        assertNull(offer)
        coVerify(exactly = 0) { userConfig.setLatestAppUpdate(any()) }
    }

    @Test
    fun `sync never reaches the network for local builds`() = runTest {
        val offer = coordinator(PlatformStore.Local).syncAppUpdate()

        assertNull(offer)
        coVerify(exactly = 0) { getRemoteConfig.getRemoteConfig() }
    }

    @Test
    fun `sync persists an older release without offering it`() = runTest {
        remoteReleases(Release(version = "0.9.0", store = PlatformStore.GooglePlay, upgradeRequired = false))

        val offer = coordinator(PlatformStore.GooglePlay).syncAppUpdate()

        assertNull(offer)
        coVerify { userConfig.setLatestAppUpdate(AppUpdateInfo(version = "0.9.0", isRequired = false)) }
    }

    @Test
    fun `sync skips an already skipped optional release`() = runTest {
        skippedVersion.value = "3.0.0"
        remoteReleases(Release(version = "3.0.0", store = PlatformStore.GooglePlay, upgradeRequired = false))

        assertNull(coordinator(PlatformStore.GooglePlay).syncAppUpdate())
    }

    @Test
    fun `sync offers a skipped release when the upgrade is required`() = runTest {
        skippedVersion.value = "3.0.0"
        remoteReleases(Release(version = "3.0.0", store = PlatformStore.GooglePlay, upgradeRequired = true))

        assertEquals("3.0.0", coordinator(PlatformStore.GooglePlay).syncAppUpdate()?.version)
    }

    @Test
    fun `observed offer disappears once the version is skipped`() = runTest {
        latestAppUpdate.value = AppUpdateInfo(version = "3.0.0", isRequired = false)
        val coordinator = coordinator(PlatformStore.ApkUniversal)

        assertEquals("3.0.0", coordinator.observeAppUpdateOffer().first()?.version)

        skippedVersion.value = "3.0.0"

        assertNull(coordinator.observeAppUpdateOffer().first())
    }

    private fun remoteReleases(vararg releases: Release) {
        coEvery { getRemoteConfig.getRemoteConfig() } returns ConfigResponse(
            releases = releases.toList(),
            versions = ConfigVersions(fiatOnRampAssets = 0, fiatOffRampAssets = 0, swapAssets = 0),
            swap = SwapConfig(enabledProviders = emptyList()),
        )
    }

    private fun coordinator(platformStore: PlatformStore) = AppUpdateCoordinator(
        getRemoteConfig = getRemoteConfig,
        userConfig = userConfig,
        buildInfo = BuildInfo(
            platformStore = platformStore,
            versionName = "1.0.0",
            versionCode = 1,
            requestPushToken = requestPushToken,
        ),
    )
}
