package com.gemwallet.android.data.repositories.device

import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class DeviceSyncCoordinatorTest {

    @Test
    fun synchronize_runsEachCallerAfterInFlightRunCompletes() = runTest {
        val coordinator = DeviceSyncCoordinator(this)
        val firstStarted = CompletableDeferred<Unit>()
        val firstRelease = CompletableDeferred<Unit>()
        val runs = mutableListOf<String>()

        launch {
            coordinator.synchronize {
                firstStarted.complete(Unit)
                firstRelease.await()
                runs += "first"
            }
        }
        firstStarted.await()

        launch {
            coordinator.synchronize {
                runs += "second"
            }
        }
        runCurrent()
        assertEquals(emptyList<String>(), runs)

        firstRelease.complete(Unit)
        advanceUntilIdle()

        assertEquals(listOf("first", "second"), runs)
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
