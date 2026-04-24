package com.gemwallet.android.ext

import com.gemwallet.android.model.GemNetworkError
import uniffi.gemstone.AlienException
import uniffi.gemstone.GatewayException
import java.io.EOFException
import java.io.IOException
import java.net.ConnectException
import java.net.NoRouteToHostException
import java.net.UnknownHostException
import java.security.cert.CertPathValidatorException
import javax.net.ssl.SSLHandshakeException

fun Throwable.toGemNetworkError(): GemNetworkError? = when (this) {
    is GatewayException.NetworkException -> GemNetworkError.Display(msg)
    is AlienException.RequestException -> GemNetworkError.Generic(msg)
    is AlienException.ResponseException -> GemNetworkError.Generic(msg)
    is IOException -> if (isNetworkUnavailable()) {
        GemNetworkError.Offline
    } else {
        GemNetworkError.Generic(toGatewayNetworkMessage())
    }
    else -> cause?.toGemNetworkError()
}

fun IOException.toGatewayNetworkMessage(
    offlineMessage: String? = null,
): String = when {
    isNetworkUnavailable() -> offlineMessage ?: message ?: toString()
    this is SSLHandshakeException -> certPathValidationMessage() ?: message ?: toString()
    else -> message ?: toString()
}

private fun IOException.isNetworkUnavailable(): Boolean {
    return this is UnknownHostException ||
        this is ConnectException ||
        this is NoRouteToHostException ||
        hasCause<EOFException>()
}

private inline fun <reified T : Throwable> Throwable.hasCause(): Boolean {
    var err: Throwable? = this
    while (err != null) {
        if (err is T) {
            return true
        }
        err = err.cause
    }
    return false
}

private fun Throwable.certPathValidationMessage(): String? {
    var err: Throwable? = this
    while (err != null) {
        if (err is CertPathValidatorException) {
            return err.message ?: err.toString()
        }
        err = err.cause
    }
    return null
}
