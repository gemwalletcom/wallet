package com.gemwallet.android.data.repositories.device

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.async
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

internal class DeviceSyncCoordinator(
    private val scope: CoroutineScope,
) {
    private val mutex = Mutex()
    private var current: Deferred<Unit> = CompletableDeferred(Unit)

    suspend fun synchronize(operation: suspend () -> Unit) {
        val task = mutex.withLock {
            val previous = current
            scope.async {
                previous.join()
                try {
                    operation()
                } catch (error: CancellationException) {
                    throw error
                } catch (_: Throwable) {
                }
            }.also { current = it }
        }
        task.await()
    }
}
