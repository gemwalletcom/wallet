package com.gemwallet.android

import kotlin.time.Duration
import kotlin.time.TimeMark
import kotlin.time.TimeSource

internal class AuthRequestQueue(private val timeout: Duration = SystemAuthPolicy.authRequestTimeout, private val timeSource: TimeSource = TimeSource.Monotonic) {
    private var nextId = 0L
    private var active: PendingAuthRequest? = null
    private var activeExpiresAt: TimeMark? = null
    private val queued = ArrayDeque<PendingAuthRequest>()

    @Synchronized
    fun enqueue(requiresConfirmation: Boolean = false, onCancel: () -> Unit = {}, onSuccess: () -> Unit): PendingAuthRequest? {
        queued.addLast(
            PendingAuthRequest(
                id = nextId++,
                requiresConfirmation = requiresConfirmation,
                onCancel = onCancel,
                onSuccess = onSuccess,
            ),
        )
        return startNext()
    }

    @Synchronized
    fun active(): PendingAuthRequest? = active

    @Synchronized
    fun hasActive(): Boolean = active != null

    @Synchronized
    fun activeRequiresConfirmation(): Boolean = active?.requiresConfirmation == true

    @Synchronized
    fun activeTimeLeft(): Duration? = activeExpiresAt?.let { -it.elapsedNow() }

    @Synchronized
    fun completeActive(): PendingAuthRequest? {
        val request = active ?: return null
        active = null
        activeExpiresAt = null
        return request
    }

    @Synchronized
    fun completeActive(requestId: Long): PendingAuthRequest? {
        if (active?.id != requestId) return null
        return completeActive()
    }

    @Synchronized
    fun startNext(): PendingAuthRequest? {
        if (active != null || queued.isEmpty()) {
            return null
        }
        activeExpiresAt = timeSource.markNow() + timeout
        return queued.removeFirst().also { active = it }
    }
}

internal data class PendingAuthRequest(val id: Long, val requiresConfirmation: Boolean = false, val onCancel: () -> Unit = {}, val onSuccess: () -> Unit)
