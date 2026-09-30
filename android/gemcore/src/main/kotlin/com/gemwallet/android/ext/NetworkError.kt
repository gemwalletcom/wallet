package com.gemwallet.android.ext

import java.io.EOFException
import java.io.IOException
import java.net.ConnectException
import java.net.NoRouteToHostException
import java.net.UnknownHostException
import java.security.cert.CertPathValidatorException
import javax.net.ssl.SSLHandshakeException

fun IOException.toGatewayNetworkMessage(): String = when {
    this is SSLHandshakeException -> certPathValidationMessage() ?: message ?: toString()
    else -> message ?: toString()
}

fun IOException.isNetworkUnavailable(): Boolean = this is UnknownHostException ||
    this is ConnectException ||
    this is NoRouteToHostException ||
    hasCause<EOFException>()

private inline fun <reified T : Throwable> Throwable.hasCause(): Boolean {
    var current: Throwable? = this
    while (current != null) {
        if (current is T) {
            return true
        }
        current = current.cause
    }
    return false
}

private fun Throwable.certPathValidationMessage(): String? {
    var current: Throwable? = this
    while (current != null) {
        if (current is CertPathValidatorException) {
            return current.message ?: current.toString()
        }
        current = current.cause
    }
    return null
}
