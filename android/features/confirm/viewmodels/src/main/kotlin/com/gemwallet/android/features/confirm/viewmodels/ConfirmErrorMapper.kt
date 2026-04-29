package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.ext.toGemNetworkError
import com.gemwallet.android.model.GemPlatformErrors
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GatewayException

internal fun Throwable.toPreloadConfirmError(chain: Chain): ConfirmError {
    if (this is GatewayException.PlatformException && msg == GemPlatformErrors.Dust.message) {
        return ConfirmError.DustThreshold(chain)
    }
    return toGemNetworkError()
        ?.let { ConfirmError.NetworkError(it) }
        ?: ConfirmError.PreloadError
}

internal fun Throwable.toBroadcastConfirmError(): ConfirmError = when (this) {
    is ConfirmError -> this
    else -> toGemNetworkError()
        ?.let { ConfirmError.NetworkError(it) }
        ?: ConfirmError.BroadcastError(message ?: toString())
}
