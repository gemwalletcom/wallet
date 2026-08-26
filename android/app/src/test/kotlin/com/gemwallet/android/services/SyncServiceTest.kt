package com.gemwallet.android.services

import com.gemwallet.android.cases.device.SyncDevice
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

class SyncServiceTest {
    private val appStartService = mockk<GemAppStartService>()
    private val syncDevice = mockk<SyncDevice>(relaxed = true)
    private val subject = SyncService(
        appStartService = appStartService,
        syncDevice = syncDevice,
    )

    @Test
    fun sync_runsAppStartThenSyncsDevice() = runBlocking {
        coEvery { appStartService.run() } returns emptyList()

        subject.sync()

        coVerify(exactly = 1) { appStartService.run() }
        coVerify(exactly = 1) { syncDevice.syncDevice() }
    }

    @Test
    fun sync_syncsDeviceEvenWhenAppStartStepsFail() = runBlocking {
        mockkStatic(Log::class)
        every { Log.e(any(), any()) } returns 0
        coEvery { appStartService.run() } returns listOf(GemAppStartFailure(GemAppStartStep.UPDATE_CONFIG, "offline"))

        subject.sync()

        coVerify(exactly = 1) { syncDevice.syncDevice() }
    }
}
