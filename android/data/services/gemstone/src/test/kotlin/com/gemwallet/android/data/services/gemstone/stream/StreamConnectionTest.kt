package com.gemwallet.android.data.services.gemstone.stream

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import java.time.Duration

class StreamConnectionTest {

    @Test
    fun latencyAsksForAFreshPing() = runTest {
        val connection = object : WebSocketConnectable {
            override val isConnected: Boolean = true
            override val connectionLatency: Duration = Duration.ofMillis(10)
            override suspend fun ping(): Duration = Duration.ofMillis(80)
            override fun connect(): Flow<WebSocketEvent> = emptyFlow()
            override suspend fun send(message: String): Boolean = true
        }

        assertEquals(Duration.ofMillis(80), GemstoneStreamConnection(connection).latency())
    }
}
