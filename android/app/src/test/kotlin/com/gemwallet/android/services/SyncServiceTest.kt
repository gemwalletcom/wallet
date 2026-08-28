package com.gemwallet.android.services

import android.util.Log
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockkStatic
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.runBlocking
import org.junit.Test
import uniffi.gemstone.GemAppStartFailure
import uniffi.gemstone.GemAppStartService
import uniffi.gemstone.GemAppStartStep
import uniffi.gemstone.GemDeviceService

class SyncServiceTest {
    private val appStartService = mockk<GemAppStartService>()
    private val deviceService = mockk<GemDeviceService>(relaxed = true)
    private val subject = SyncService(
        appStartService = appStartService,
        deviceService = deviceService,
    )

    @Test
    fun sync_runsAppStartThenSyncsDevice() = runBlocking {
        coEvery { appStartService.run() } returns emptyList()

        subject.sync()

        coVerify(exactly = 1) { appStartService.run() }
        coVerify(exactly = 1) { deviceService.synchronizeIfNeeded() }
    }

    @Test
    fun sync_syncsDeviceEvenWhenAppStartStepsFail() = runBlocking {
        mockkStatic(Log::class)
        every { Log.e(any(), any()) } returns 0
        coEvery { appStartService.run() } returns listOf(GemAppStartFailure(GemAppStartStep.UPDATE_CONFIG, "offline"))

        subject.sync()

        coVerify(exactly = 1) { deviceService.synchronizeIfNeeded() }
    }
}
