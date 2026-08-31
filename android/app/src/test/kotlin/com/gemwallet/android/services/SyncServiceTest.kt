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

class SyncServiceTest {
    private val appStartService = mockk<GemAppStartService>()
    private val subject = SyncService(appStartService = appStartService)

    @Test
    fun sync_runsAppStart() = runBlocking {
        coEvery { appStartService.run() } returns emptyList()

        subject.sync()

        coVerify(exactly = 1) { appStartService.run() }
    }

    @Test
    fun sync_reportsFailedStepsWithoutThrowing() = runBlocking {
        mockkStatic(Log::class)
        every { Log.e(any(), any()) } returns 0
        coEvery { appStartService.run() } returns listOf(GemAppStartFailure(GemAppStartStep.UPDATE_CONFIG, "offline"))

        subject.sync()

        coVerify(exactly = 1) { appStartService.run() }
    }
}
