package com.gemwallet.android.data.coordinators.update

import com.gemwallet.android.model.AppUpdateChannel
import com.gemwallet.android.testkit.mockAppUpdateOffer
import com.gemwallet.android.testkit.mockBuildInfo
import com.wallet.core.primitives.PlatformStore
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemAppUpdateOffer
import uniffi.gemstone.GemAppUpdateService

class AppUpdateCoordinatorTest {

    @Test
    fun `sync offers the release through the store channel`() = runTest {
        val coordinator = coordinator(PlatformStore.GooglePlay, GemAppUpdateOffer("3.0.0", true))

        val offer = coordinator.syncAppUpdate()

        assertEquals("3.0.0", offer?.version)
        assertEquals(AppUpdateChannel.Store, offer?.channel)
        assertEquals(offer, coordinator.observeAppUpdateOffer().first())
    }

    @Test
    fun `sync offers the universal apk build through the in app channel`() = runTest {
        val offer = coordinator(PlatformStore.ApkUniversal, GemAppUpdateOffer("3.0.0", true)).syncAppUpdate()

        assertEquals(AppUpdateChannel.InAppApk, offer?.channel)
    }

    @Test
    fun `skip clears the observed offer once core stops offering it`() = runTest {
        val appUpdateService = mockk<GemAppUpdateService>()
        val offers = mutableListOf<GemAppUpdateOffer?>(GemAppUpdateOffer("3.0.0", true), null)
        coEvery { appUpdateService.check(any(), any()) } answers { offers.removeAt(0) }
        every { appUpdateService.skip(any()) } returns Unit
        val coordinator = AppUpdateCoordinator(appUpdateService, mockBuildInfo())

        coordinator.syncAppUpdate()
        coordinator.skipAppUpdate(mockAppUpdateOffer(canSkip = true))

        assertNull(coordinator.observeAppUpdateOffer().first())
    }

    private fun coordinator(platformStore: PlatformStore, update: GemAppUpdateOffer?): AppUpdateCoordinator {
        val appUpdateService = mockk<GemAppUpdateService> {
            coEvery { check(any(), any()) } returns update
        }
        return AppUpdateCoordinator(appUpdateService, mockBuildInfo(platformStore))
    }
}
