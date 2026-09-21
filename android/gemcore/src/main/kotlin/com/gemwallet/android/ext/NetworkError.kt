package com.gemwallet.android.ext

import uniffi.gemstone.AlienException
import uniffi.gemstone.GatewayException
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.alienErrorText
import java.io.EOFException
import java.io.IOException
import java.net.ConnectException
import java.net.NoRouteToHostException
import java.net.UnknownHostException
import java.security.cert.CertPathValidatorException
import javax.net.ssl.SSLHandshakeException

fun Throwable.toGemErrorText(): GemErrorText? = when (this) {
    is GatewayException -> text()

    is AlienException -> alienErrorText(this)

    is IOException -> if (isNetworkUnavailable()) {
        GemErrorText.NetworkOffline
    } else {
        GemErrorText.NetworkMessage(toGatewayNetworkMessage())
    }

    else -> cause?.toGemErrorText()
}

fun IOException.toGatewayNetworkMessage(): String = when {
    this is SSLHandshakeException -> certPathValidationMessage() ?: message ?: toString()
    else -> message ?: toString()
}

fun IOException.isNetworkUnavailable(): Boolean = this is UnknownHostException ||
    this is ConnectException ||
    this is NoRouteToHostException ||
    hasCause<EOFException>()

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
