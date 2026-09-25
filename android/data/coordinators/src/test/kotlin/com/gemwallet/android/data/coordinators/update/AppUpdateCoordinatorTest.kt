package com.gemwallet.android.data.coordinators.update

import com.gemwallet.android.testkit.mockAppUpdateOffer
import com.gemwallet.android.testkit.mockBuildInfo
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
    fun `sync offers the release core returned`() = runTest {
        val update = mockAppUpdateOffer(apkUrl = "https://apk.gemwallet.com/gem_wallet_universal_2.0.0.apk")
        val appUpdateService = mockk<GemAppUpdateService> {
            coEvery { check(any(), any()) } returns update
        }
        val coordinator = AppUpdateCoordinator(appUpdateService, mockBuildInfo())

        assertEquals(update, coordinator.syncAppUpdate())
        assertEquals(update, coordinator.observeAppUpdateOffer().first())
    }

    @Test
    fun `skip clears the observed offer once core stops offering it`() = runTest {
        val appUpdateService = mockk<GemAppUpdateService>()
        val offers = mutableListOf<GemAppUpdateOffer?>(mockAppUpdateOffer(), null)
        coEvery { appUpdateService.check(any(), any()) } answers { offers.removeAt(0) }
        every { appUpdateService.skip(any()) } returns Unit
        val coordinator = AppUpdateCoordinator(appUpdateService, mockBuildInfo())

        coordinator.syncAppUpdate()
        coordinator.skipAppUpdate(mockAppUpdateOffer(canSkip = true))

        assertNull(coordinator.observeAppUpdateOffer().first())
    }
}
