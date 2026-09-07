package com.gemwallet.android.data.coordinators.update

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AppUpdateChannel
import com.gemwallet.android.model.BuildInfo
import com.gemwallet.android.application.device.cases.RequestPushToken
import com.wallet.core.primitives.PlatformStore
import com.wallet.core.primitives.Release
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemAppUpdateService

class AppUpdateCoordinatorTest {

    @Test
    fun `sync offers the release through the store channel`() = runTest {
        val coordinator = coordinator(PlatformStore.GooglePlay, release("3.0.0"))

        val offer = coordinator.syncAppUpdate()

        assertEquals("3.0.0", offer?.version)
        assertEquals(AppUpdateChannel.Store, offer?.channel)
        assertEquals(offer, coordinator.observeAppUpdateOffer().first())
    }

    @Test
    fun `sync offers the universal apk build through the in app channel`() = runTest {
        val offer = coordinator(PlatformStore.ApkUniversal, release("3.0.0")).syncAppUpdate()

        assertEquals(AppUpdateChannel.InAppApk, offer?.channel)
    }

    @Test
    fun `skip clears the observed offer once core stops offering it`() = runTest {
        val appUpdateService = mockk<GemAppUpdateService>()
        val releases = mutableListOf<Release?>(release("3.0.0"), null)
        coEvery { appUpdateService.check(any(), any()) } answers { releases.removeAt(0)?.toGem() }
        every { appUpdateService.skip(any()) } returns Unit
        val coordinator = AppUpdateCoordinator(appUpdateService, buildInfo(PlatformStore.GooglePlay))

        coordinator.syncAppUpdate()
        coordinator.skipAppUpdate("3.0.0")

        assertNull(coordinator.observeAppUpdateOffer().first())
    }

    private fun release(version: String) = Release(version = version, store = PlatformStore.GooglePlay, upgradeRequired = false)

    private fun coordinator(platformStore: PlatformStore, release: Release?): AppUpdateCoordinator {
        val appUpdateService = mockk<GemAppUpdateService> {
            coEvery { check(any(), any()) } returns release?.toGem()
        }
        return AppUpdateCoordinator(appUpdateService, buildInfo(platformStore))
    }

    private fun buildInfo(platformStore: PlatformStore) = BuildInfo(
        platformStore = platformStore,
        versionName = "1.0.0",
        versionCode = 1,
        requestPushToken = mockk<RequestPushToken>(relaxed = true),
    )
}
