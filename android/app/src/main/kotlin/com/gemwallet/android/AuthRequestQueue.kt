package com.gemwallet.android

internal class AuthRequestQueue {
    private var nextId = 0L
    private var active: PendingAuthRequest? = null
    private val queued = ArrayDeque<PendingAuthRequest>()

    @Synchronized
    fun enqueue(requiresConfirmation: Boolean = false, onSuccess: () -> Unit): PendingAuthRequest? {
        queued.addLast(
            PendingAuthRequest(
                id = nextId++,
                requiresConfirmation = requiresConfirmation,
                onSuccess = onSuccess,
            )
        )
        return startNext()
    }

    @Synchronized
    fun hasActive(): Boolean = active != null

    @Synchronized
    fun activeRequiresConfirmation(): Boolean = active?.requiresConfirmation == true

    @Synchronized
    fun completeActive(): PendingAuthRequest? {
        val request = active ?: return null
        active = null
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
        return queued.removeFirst().also { active = it }
    }
}

internal data class PendingAuthRequest(
    val id: Long,
    val requiresConfirmation: Boolean = false,
    val onSuccess: () -> Unit,
)
