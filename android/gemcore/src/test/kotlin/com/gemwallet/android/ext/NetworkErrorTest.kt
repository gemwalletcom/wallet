package com.gemwallet.android.ext

import com.gemwallet.android.model.GemNetworkError
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.AlienException
import uniffi.gemstone.GatewayException
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
                GemNetworkError.Display("Network error: certificate path failed"),
            AlienException.RequestException("request failed") to GemNetworkError.Generic("request failed"),
            AlienException.ResponseException("response failed") to GemNetworkError.Generic("response failed"),
            IOException("unexpected end of stream", EOFException()) to GemNetworkError.Offline,
            IOException("unexpected end of stream on https://gemnodes.com/...") to
                GemNetworkError.Generic("unexpected end of stream on https://gemnodes.com/..."),
            SocketTimeoutException("timeout") to GemNetworkError.Generic("timeout"),
            IllegalStateException("outer", AlienException.RequestException("request failed")) to
                GemNetworkError.Generic("request failed"),
        ).forEach { (source, expected) ->
            assertEquals(expected, source.toGemNetworkError())
        }

        assertNull(IllegalStateException("bad state").toGemNetworkError())
    }

    @Test
    fun mapsOfflineConnectionErrors() {
        val offline = "The Internet connection appears to be offline."

        listOf(
            UnknownHostException("api.example.com"),
            ConnectException("failed to connect"),
            NoRouteToHostException("no route to host"),
            IOException("unexpected end of stream", EOFException()),
        ).forEach { assertEquals(offline, it.toGatewayNetworkMessage(offline)) }
    }

    @Test
    fun mapsSslCertificateErrorToReadableMessage() {
        val source = SSLHandshakeException(
            "java.security.cert.CertPathValidatorException: Trust anchor for certification path not found."
        ).apply {
            initCause(CertPathValidatorException("Trust anchor for certification path not found."))
        }

        assertEquals("Trust anchor for certification path not found.", source.toGatewayNetworkMessage())
    }
}
