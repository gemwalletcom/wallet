package com.gemwallet.android.ext

import uniffi.gemstone.AlienException
import uniffi.gemstone.GatewayException
import uniffi.gemstone.GemAddNodeException
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemPaymentException
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletConnectException
import uniffi.gemstone.GemstoneException
import uniffi.gemstone.alienErrorText
import uniffi.gemstone.paymentErrorText
import java.io.IOException

fun Throwable.errorText(): GemErrorText = errorTextOrNull() ?: GemErrorText.Message(message ?: toString())

fun Throwable.errorTextOrNull(): GemErrorText? = when (this) {
    is GemServiceException -> text()

    is GatewayException -> text()

    is GemAddNodeException -> text()

    is GemWalletConnectException -> text()

    is GemstoneException -> text()

    is GemPaymentException -> paymentErrorText(this)

    is AlienException -> alienErrorText(this)

    is IOException -> if (isNetworkUnavailable()) {
        GemErrorText.NetworkOffline
    } else {
        GemErrorText.NetworkMessage(toGatewayNetworkMessage())
    }

    else -> cause?.errorTextOrNull()
}
