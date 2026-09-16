package com.gemwallet.android.ext

import uniffi.gemstone.GemAddNodeException
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletConnectException
import uniffi.gemstone.GemstoneException
import uniffi.gemstone.GatewayException

fun Throwable.errorText(): GemErrorText = when (this) {
    is GemServiceException -> text()
    is GatewayException -> text()
    is GemAddNodeException -> text()
    is GemWalletConnectException -> text()
    is GemstoneException -> text()
    else -> GemErrorText.Message(message ?: toString())
}
