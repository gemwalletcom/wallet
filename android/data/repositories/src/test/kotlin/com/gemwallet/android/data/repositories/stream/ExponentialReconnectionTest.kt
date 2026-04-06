package com.gemwallet.android.data.repositories.stream

import org.junit.Assert.assertEquals
import org.junit.Test

class ExponentialReconnectionTest {

    @Test
    fun `reconnectAfterMs grows exponentially and caps at maxDelay`() {
        val reconnection = ExponentialReconnection()

        assertEquals(300L, reconnection.reconnectAfterMs(0))
        assertEquals(6025L, reconnection.reconnectAfterMs(3))
        assertEquals(60_000L, reconnection.reconnectAfterMs(100))
    }
}
