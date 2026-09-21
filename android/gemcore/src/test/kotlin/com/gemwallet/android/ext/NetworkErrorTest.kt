package com.gemwallet.android.ext

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.AlienException
import uniffi.gemstone.GatewayException
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemPaymentException
import java.io.EOFException
import java.io.IOException
import java.net.ConnectException
import java.net.NoRouteToHostException
import java.net.SocketTimeoutException
import java.net.UnknownHostException
import java.security.cert.CertPathValidatorException
import javax.net.ssl.SSLHandshakeException

class NetworkErrorTest {

    @Test
    fun mapsNetworkErrors() {
        listOf(
            GatewayException.NetworkException("Network error: certificate path failed") to
                GemErrorText.Message("Network error: certificate path failed"),
            AlienException.RequestException("request failed") to GemErrorText.NetworkMessage("request failed"),
            AlienException.Offline() to GemErrorText.NetworkOffline,
            GatewayException.Offline() to GemErrorText.NetworkOffline,
            AlienException.ResponseException("response failed") to GemErrorText.NetworkMessage("response failed"),
            IOException("unexpected end of stream", EOFException()) to GemErrorText.NetworkOffline,
            IOException("unexpected end of stream on https://gemnodes.com/...") to
                GemErrorText.NetworkMessage("unexpected end of stream on https://gemnodes.com/..."),
            SocketTimeoutException("timeout") to GemErrorText.NetworkMessage("timeout"),
            IllegalStateException("outer", AlienException.RequestException("request failed")) to
                GemErrorText.NetworkMessage("request failed"),
        ).forEach { (source, expected) ->
            assertEquals(expected, source.errorTextOrNull())
        }

        assertNull(IllegalStateException("bad state").errorTextOrNull())
        assertEquals(GemErrorText.Message("bad state"), IllegalStateException("bad state").errorText())
        assertEquals("a scan with no payment option reads the same sentence on both apps", GemErrorText.NotSupported, GemPaymentException.NoPaymentOptions().errorText())
    }

    @Test
    fun mapsOfflineConnectionErrors() {
        listOf(
            UnknownHostException("api.example.com"),
            ConnectException("failed to connect"),
            NoRouteToHostException("no route to host"),
            IOException("unexpected end of stream", EOFException()),
        ).forEach { assertTrue(it.isNetworkUnavailable()) }
        assertFalse(SocketTimeoutException("timeout").isNetworkUnavailable())
    }

    @Test
    fun mapsSslCertificateErrorToReadableMessage() {
        val source = SSLHandshakeException(
            "java.security.cert.CertPathValidatorException: Trust anchor for certification path not found.",
        ).apply {
            initCause(CertPathValidatorException("Trust anchor for certification path not found."))
        }

        assertEquals("Trust anchor for certification path not found.", source.toGatewayNetworkMessage())
    }
}
