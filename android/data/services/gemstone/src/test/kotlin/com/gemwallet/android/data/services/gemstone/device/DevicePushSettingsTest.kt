package com.gemwallet.android.data.services.gemstone.device

import android.content.Context
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import dagger.Lazy
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.coVerifyOrder
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemNotificationsService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemPushResult
import uniffi.gemstone.GemPushState

@OptIn(ExperimentalCoroutinesApi::class)
class DevicePushSettingsTest {

    private val deviceService = mockk<GemDeviceService>(relaxed = true)
    private val notificationsService = mockk<GemNotificationsService>(relaxed = true)
    private val userConfig = mockk<UserConfig>(relaxed = true)

    @Test
    fun `switching push asks for permission before it records the ask`() = runTest {
        settings(ConfigStore(mockk(relaxed = true))).switchPushEnabled(true)

        coVerifyOrder {
            notificationsService.setEnabled(true)
            userConfig.stopAskNotifications()
        }
    }

    @Test
    fun `a new token is stored and pushed to the backend`() = runTest {
        val configStore = mockk<ConfigStore>(relaxed = true) {
            every { getString("push_token", any()) } returns "old-token"
        }
        val subject = settings(configStore)

        subject.setPushToken("new-token")
        advanceUntilIdle()

        verify(exactly = 1) { configStore.putString("push_token", "new-token") }
        coVerify(exactly = 1) { deviceService.synchronizeIfNeeded() }
    }

    @Test
    fun `the same token is not rewritten and does not push again`() = runTest {
        val configStore = mockk<ConfigStore>(relaxed = true) {
            every { getString("push_token", any()) } returns "same-token"
        }
        val subject = settings(configStore)

        subject.setPushToken("same-token")
        advanceUntilIdle()

        verify(exactly = 0) { configStore.putString(any(), any(), any()) }
        coVerify(exactly = 0) { deviceService.synchronizeIfNeeded() }
    }

    @Test
    fun `the toggle reads the state Core answers with, not the one it asked for`() = runTest {
        val subject = settings(mockk<ConfigStore>(relaxed = true))
        coEvery { notificationsService.setEnabled(true) } returns GemPushState(isEnabled = false, result = GemPushResult.PermissionDenied)

        val state = subject.switchPushEnabled(true)
        advanceUntilIdle()

        assertEquals(false, state.isEnabled)
        assertEquals(GemPushResult.PermissionDenied, state.result)
        coVerify(exactly = 1) { notificationsService.setEnabled(true) }
    }

    @Test
    fun `a failed registration is carried back to the screen`() = runTest {
        val subject = settings(mockk<ConfigStore>(relaxed = true))
        coEvery { notificationsService.setEnabled(true) } returns GemPushState(isEnabled = true, result = GemPushResult.NotRegistered(GemErrorText.NetworkOffline))

        val state = subject.switchPushEnabled(true)

        assertEquals("the preference Core stored is what the toggle shows", true, state.isEnabled)
        assertEquals(GemPushResult.NotRegistered(GemErrorText.NetworkOffline), state.result)
    }

    private fun TestScope.settings(configStore: ConfigStore) = DevicePushSettings(
        context = mockk<Context>(relaxed = true),
        configStore = configStore,
        notificationsAvailable = true,
        preferencesService = mockk<GemPreferencesService>(relaxed = true),
        deviceService = mockk<Lazy<GemDeviceService>> { every { get() } returns deviceService },
        notificationsService = mockk<Lazy<GemNotificationsService>> { every { get() } returns notificationsService },
        userConfig = userConfig,
        scope = this,
    )
}
