package com.gemwallet.android.application.notifications

import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow

class NotificationPermissionRequests {

    private val _requests = MutableSharedFlow<CompletableDeferred<Boolean>>(extraBufferCapacity = 1)
    val requests: SharedFlow<CompletableDeferred<Boolean>> = _requests.asSharedFlow()

    suspend fun request(): Boolean {
        val granted = CompletableDeferred<Boolean>()
        if (!_requests.tryEmit(granted)) {
            return false
        }
        return granted.await()
    }
}
