package com.gemwallet.android.data.services.gemstone.device

import android.content.Context
import androidx.core.app.NotificationManagerCompat
import com.gemwallet.android.application.device.cases.RequestPushToken
import com.gemwallet.android.data.service.store.ConfigStore
import com.wallet.core.primitives.PlatformStore
import dagger.Lazy
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import io.mockk.verify
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Assert.assertFalse
import org.junit.Test
import uniffi.gemstone.GemDeviceKeyService
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemPreferencesService

@OptIn(ExperimentalCoroutinesApi::class)
class DevicePlatformTest {

    @Test
    fun emptyRecoveredToken_doesNotSynchronizeAgain() = runTest {
        val configStore = mockk<ConfigStore> {
            every { getString("push_token", any()) } returns ""
        }
        val deviceService = mockk<GemDeviceService>(relaxed = true)
        val lazyDeviceService = mockk<Lazy<GemDeviceService>> {
            every { get() } returns deviceService
        }
        val requestPushToken = mockk<RequestPushToken> {
            coEvery { requestToken(any()) } answers {
                firstArg<(String) -> Unit>()("")
            }
        }
        val subject = GemstoneDevicePlatform(
            context = mockk<Context>(relaxed = true),
            deviceService = lazyDeviceService,
            configStore = configStore,
            requestPushToken = requestPushToken,
            platformStore = PlatformStore.GooglePlay,
            notificationsAvailable = true,
            versionName = "1.0",
            deviceKeyService = mockk<GemDeviceKeyService>(relaxed = true),
            preferencesService = mockk<GemPreferencesService>(relaxed = true),
            scope = this,
        )

        assertEquals("", subject.pushToken())
        advanceUntilIdle()

        coVerify(exactly = 1) { requestPushToken.requestToken(any()) }
        verify(exactly = 0) { configStore.putString(any(), any(), any()) }
        coVerify(exactly = 0) { deviceService.synchronizeIfNeeded() }
    }

    @Test
    fun `push is not enabled while the system notification permission is revoked`() = runTest {
        mockkStatic(NotificationManagerCompat::class)
        try {
            val manager = mockk<NotificationManagerCompat>()
            every { NotificationManagerCompat.from(any()) } returns manager
            val preferencesService = mockk<GemPreferencesService>(relaxed = true) {
                every { isPushNotificationsEnabled() } returns true
            }
            val subject = platform(preferencesService)

            every { manager.areNotificationsEnabled() } returns true
            assertTrue(subject.isPushEnabled())

            every { manager.areNotificationsEnabled() } returns false
            assertFalse(subject.isPushEnabled())
        } finally {
            unmockkStatic(NotificationManagerCompat::class)
        }
    }

    private fun TestScope.platform(preferencesService: GemPreferencesService) = GemstoneDevicePlatform(
        context = mockk<Context>(relaxed = true),
        deviceService = mockk<Lazy<GemDeviceService>>(relaxed = true),
        configStore = mockk<ConfigStore>(relaxed = true),
        requestPushToken = mockk<RequestPushToken>(relaxed = true),
        platformStore = PlatformStore.GooglePlay,
        notificationsAvailable = true,
        versionName = "1.0",
        deviceKeyService = mockk<GemDeviceKeyService>(relaxed = true),
        preferencesService = preferencesService,
        scope = this,
    )
}
