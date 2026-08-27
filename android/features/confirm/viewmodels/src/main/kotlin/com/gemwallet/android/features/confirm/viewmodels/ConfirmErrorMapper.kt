package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.ext.toGemNetworkError
import com.gemwallet.android.model.GemNetworkError
import uniffi.gemstone.GemConfirmException

internal fun Throwable.toPreloadConfirmError(): ConfirmError = when (this) {
    is ConfirmError -> this
    is GemConfirmException.ScanMalicious -> ConfirmError.ScanTransactionMalicious
    is GemConfirmException.ScanMemoRequired -> ConfirmError.ScanTransactionMemoRequired(symbol)
    is GemConfirmException.Network -> ConfirmError.NetworkError(GemNetworkError.Display(msg))
    else -> toGemNetworkError()
        ?.let { ConfirmError.NetworkError(it) }
        ?: ConfirmError.PreloadError
}

internal fun Throwable.toBroadcastConfirmError(): ConfirmError = when (this) {
    is ConfirmError -> this
    is GemConfirmException.Network -> ConfirmError.NetworkError(GemNetworkError.Display(msg))
    else -> toGemNetworkError()
        ?.let { ConfirmError.NetworkError(it) }
        ?: ConfirmError.BroadcastError(message ?: toString())
}
