package com.gemwallet.android.data.repositories.device

import kotlinx.coroutines.delay
import kotlinx.coroutines.joinAll
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class DeviceSyncCoordinatorTest {

    @Test
    fun synchronize_concurrentCalls_neverOverlap() = runTest {
        val coordinator = DeviceSyncCoordinator()
        var running = 0
        var maxRunning = 0
        var completed = 0

        List(4) {
            launch {
                coordinator.synchronize {
                    running += 1
                    maxRunning = maxOf(maxRunning, running)
                    delay(100)
                    running -= 1
                    completed += 1
                }
            }
        }.joinAll()

        assertEquals(1, maxRunning)
        assertEquals(4, completed)
    }

    @Test
    fun synchronize_failedPass_releasesLockForNextCaller() = runTest {
        val coordinator = DeviceSyncCoordinator()
        var completed = 0

        runCatching { coordinator.synchronize { throw IllegalStateException("sync failed") } }
        coordinator.synchronize { completed += 1 }

        assertEquals(1, completed)
    }
}
