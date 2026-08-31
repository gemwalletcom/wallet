package com.gemwallet.android.data.services.gemstone.device

import android.content.Context
import com.gemwallet.android.data.service.store.ConfigStore
import dagger.Lazy
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Test
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemPreferencesService

@OptIn(ExperimentalCoroutinesApi::class)
class DevicePushSettingsTest {

    private val deviceService = mockk<GemDeviceService>(relaxed = true)

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

    private fun TestScope.settings(configStore: ConfigStore) = DevicePushSettings(
        context = mockk<Context>(relaxed = true),
        configStore = configStore,
        notificationsAvailable = true,
        preferencesService = mockk<GemPreferencesService>(relaxed = true),
        deviceService = mockk<Lazy<GemDeviceService>> { every { get() } returns deviceService },
        scope = this,
    )
}
