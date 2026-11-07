package com.gemwallet.android.data.coordinators.wallet_connect

import com.gemwallet.android.data.services.gemstone.stream.WebSocketEvent
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemServiceException
import kotlin.time.Duration.Companion.seconds

@OptIn(ExperimentalCoroutinesApi::class)
class WalletConnectRelayTest {

    private val socket = WebSocketConnectionMock()
    private val relay = WalletConnectRelay(socket)

    @Test
    fun `a frame sent while the socket opens goes out once it is open`() = runTest {
        backgroundScope.launch { relay.connect().collect() }
        val sending = async { relay.send("frame") }
        runCurrent()

        socket.isOpen = true
        socket.events.emit(WebSocketEvent.Connected)
        sending.await()

        assertEquals(listOf("frame"), socket.sent)
    }

    @Test
    fun `a frame that finds no open socket in time fails as offline`() = runTest {
        val sending = async { runCatching { relay.send("frame") } }

        advanceTimeBy(11.seconds)

        assertTrue(sending.await().exceptionOrNull() is GemServiceException.Offline)
        assertEquals(emptyList<String>(), socket.sent)
    }
}
