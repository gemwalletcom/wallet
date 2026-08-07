package com.gemwallet.android.data.repositories.device

import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class DeviceSyncCoordinatorTest {

    @Test
    fun synchronize_joinsConcurrentCallersIntoOneRun() = runTest {
        val coordinator = DeviceSyncCoordinator(this)
        var runs = 0

        repeat(3) {
            launch {
                coordinator.synchronize {
                    delay(100)
                    runs += 1
                }
            }
        }
        advanceUntilIdle()

        assertEquals(1, runs)
    }

    @Test
    fun synchronize_runsAgainAfterPreviousCompletes() = runTest {
        val coordinator = DeviceSyncCoordinator(this)
        var runs = 0

        coordinator.synchronize { runs += 1 }
        coordinator.synchronize { runs += 1 }

        assertEquals(2, runs)
    }

    @Test
    fun synchronize_keepsFailureFromReachingCallers() = runTest {
        val coordinator = DeviceSyncCoordinator(this)
        var runs = 0

        coordinator.synchronize { error("network") }
        coordinator.synchronize { runs += 1 }

        assertEquals(1, runs)
    }
}
