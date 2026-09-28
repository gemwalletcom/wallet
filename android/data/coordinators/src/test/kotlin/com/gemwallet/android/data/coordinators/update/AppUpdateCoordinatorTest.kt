package com.gemwallet.android.data.coordinators.update

import com.gemwallet.android.testkit.mockBuildInfo
import com.gemwallet.android.testkit.mockGemAppUpdateOffer
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemAppUpdateAction
import uniffi.gemstone.GemAppUpdateService

class AppUpdateCoordinatorTest {

    @Test
    fun `sync offers the release core returned`() = runTest {
        val update = mockGemAppUpdateOffer(actions = listOf(GemAppUpdateAction.SKIP, GemAppUpdateAction.UPDATE), apkUrl = "https://apk.gemwallet.com/gem_wallet_universal_2.0.0.apk")
        val appUpdateService = mockk<GemAppUpdateService> {
            coEvery { check(any(), any()) } returns update
        }
        val coordinator = AppUpdateCoordinator(appUpdateService, mockBuildInfo())

        assertEquals(update, coordinator.syncAppUpdate())
        assertEquals(update, coordinator.observeAppUpdateOffer().first())
    }

    @Test
    fun `skip saves the version and clears the offer without checking again`() = runTest {
        val update = mockGemAppUpdateOffer(version = "2.0.0", actions = listOf(GemAppUpdateAction.SKIP, GemAppUpdateAction.UPDATE))
        val appUpdateService = mockk<GemAppUpdateService> {
            coEvery { check(any(), any()) } returns update
            every { skip(any()) } returns Unit
        }
        val coordinator = AppUpdateCoordinator(appUpdateService, mockBuildInfo())

        coordinator.syncAppUpdate()
        coordinator.skipAppUpdate(update)

        assertNull(coordinator.observeAppUpdateOffer().first())
        verify { appUpdateService.skip(update) }
        coVerify(exactly = 1) { appUpdateService.check(any(), any()) }
    }
}
