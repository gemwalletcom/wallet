package com.gemwallet.android.data.services.gemstone.device

import com.gemwallet.android.data.services.store.ConfigStore
import dagger.Lazy
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemNotificationsService
import uniffi.gemstone.GemPushResult
import uniffi.gemstone.GemPushState

@OptIn(ExperimentalCoroutinesApi::class)
class DevicePushSettingsTest {

    private val deviceService = mockk<GemDeviceService>(relaxed = true)
    private val notificationsService = mockk<GemNotificationsService>(relaxed = true)

    @Test
    fun `a new wallet asks Core and keeps the state it answered`() = runTest {
        every { notificationsService.isEnabled() } returns false
        coEvery { notificationsService.askToEnable() } returns GemPushState(isEnabled = true, result = GemPushResult.Stored)
        val subject = settings(ConfigStore(mockk(relaxed = true)))
        val shown = shownPushEnabled(subject)

        subject.enablePushForNewWallet()
        advanceUntilIdle()

        coVerify(exactly = 1) { notificationsService.askToEnable() }
        assertEquals("the toggle shows the state Core answered", listOf(false, true), shown)
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
        every { notificationsService.isEnabled() } returns true
        val subject = settings(mockk<ConfigStore>(relaxed = true))
        val shown = shownPushEnabled(subject)
        coEvery { notificationsService.setEnabled(true) } returns GemPushState(isEnabled = false, result = GemPushResult.PermissionDenied)

        val state = subject.switchPushEnabled(true)

        assertEquals(false, state.isEnabled)
        assertEquals(GemPushResult.PermissionDenied, state.result)
        assertEquals("the toggle keeps the state Core answered", listOf(true, false), shown)
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

    @Test
    fun `a return to the app shows what Core reads now`() = runTest {
        every { notificationsService.isEnabled() } returns true
        val subject = settings(mockk<ConfigStore>(relaxed = true))
        val shown = shownPushEnabled(subject)

        coEvery { notificationsService.refresh() } returns false
        subject.refreshPushEnabled()

        assertEquals("a permission revoked in system settings turns the toggle off", listOf(true, false), shown)
    }

    private fun TestScope.shownPushEnabled(subject: DevicePushSettings): List<Boolean> = mutableListOf<Boolean>().also { shown ->
        backgroundScope.launch(UnconfinedTestDispatcher(testScheduler)) { subject.getPushEnabled().toList(shown) }
    }

    private fun TestScope.settings(configStore: ConfigStore) = DevicePushSettings(
        configStore = configStore,
        notificationsAvailable = true,
        deviceService = mockk<Lazy<GemDeviceService>> { every { get() } returns deviceService },
        notificationsService = mockk<Lazy<GemNotificationsService>> { every { get() } returns notificationsService },
        ioDispatcher = UnconfinedTestDispatcher(testScheduler),
        scope = this,
    )
}
