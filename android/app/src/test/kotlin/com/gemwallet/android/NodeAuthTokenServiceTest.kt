package com.gemwallet.android

import com.gemwallet.android.cases.device.IsDeviceRegistered
import com.gemwallet.android.data.services.gemapi.DeviceToken
import com.gemwallet.android.serializer.toJson
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.test.runTest
import uniffi.gemstone.GemDeviceService
import org.junit.Test
import uniffi.gemstone.GemNodeAuthConfig
import uniffi.gemstone.GemPreferencesStore

class NodeAuthTokenServiceTest {
    private val deviceService = mockk<GemDeviceService>()
    private val isDeviceRegistered = mockk<IsDeviceRegistered>()
    private val preferences = mockk<GemPreferencesStore>(relaxed = true)
    private val config = GemNodeAuthConfig(checkIntervalSeconds = 60u, refreshThresholdSeconds = 300u)

    @Test
    fun updateIfNeededRefreshesMissingOrExpiringToken() = runTest {
        coEvery { isDeviceRegistered.isDeviceRegistered() } returns true
        every { preferences.get(any()) } returns null
        coEvery { deviceService.getToken() } returns DeviceToken("new", 1_000u).toJson()
        val service = NodeAuthTokenService(
            deviceService = deviceService,
            isDeviceRegistered = isDeviceRegistered,
            preferences = preferences,
            scope = this,
            currentTimeSeconds = { 100u },
            config = config,
        )

        service.updateIfNeeded()

        verify { preferences.set(any(), DeviceToken("new", 1_000u).toJson()) }
        coVerify(exactly = 1) { deviceService.getToken() }
    }

    @Test
    fun updateIfNeededKeepsTokenWithMoreThanFiveMinutesRemaining() = runTest {
        every { preferences.get(any()) } returns DeviceToken("current", 401u).toJson()
        coEvery { isDeviceRegistered.isDeviceRegistered() } returns true
        val service = NodeAuthTokenService(
            deviceService = deviceService,
            isDeviceRegistered = isDeviceRegistered,
            preferences = preferences,
            scope = this,
            currentTimeSeconds = { 100u },
            config = config,
        )

        service.updateIfNeeded()

        coVerify(exactly = 0) { deviceService.getToken() }
    }

    @Test
    fun updateIfNeededWaitsForDeviceRegistration() = runTest {
        coEvery { isDeviceRegistered.isDeviceRegistered() } returns false
        val service = NodeAuthTokenService(
            deviceService = deviceService,
            isDeviceRegistered = isDeviceRegistered,
            preferences = preferences,
            scope = this,
            config = config,
        )

        service.updateIfNeeded()
        coVerify(exactly = 0) { deviceService.getToken() }
    }
}
