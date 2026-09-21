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
