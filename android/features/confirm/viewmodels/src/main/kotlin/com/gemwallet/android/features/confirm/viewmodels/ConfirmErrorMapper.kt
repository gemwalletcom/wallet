package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.ext.toGemNetworkError

internal fun Throwable.toPreloadConfirmError(): ConfirmError =
    toGemNetworkError()
        ?.let { ConfirmError.NetworkError(it) }
        ?: ConfirmError.PreloadError

internal fun Throwable.toBroadcastConfirmError(): ConfirmError = when (this) {
    is ConfirmError -> this
    else -> toGemNetworkError()
        ?.let { ConfirmError.NetworkError(it) }
        ?: ConfirmError.BroadcastError(message ?: toString())
}
