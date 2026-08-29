package com.gemwallet.android.data.repositories.perpetual

import org.junit.Assert.assertEquals
import org.junit.Test

class HyperliquidObserverServiceTest {

    @Test
    fun `toWebSocketUrl converts scheme and appends ws path once`() {
        assertEquals("wss://rpc.hypercore.dev/ws", "https://rpc.hypercore.dev".toWebSocketUrl())
        assertEquals("wss://rpc.hypercore.dev/ws", "https://rpc.hypercore.dev/".toWebSocketUrl())
        assertEquals("wss://rpc.hypercore.dev/ws", "https://rpc.hypercore.dev/ws".toWebSocketUrl())
        assertEquals("ws://localhost:8545/ws", "http://localhost:8545".toWebSocketUrl())
        assertEquals("wss://api.hyperliquid.xyz/ws", "wss://api.hyperliquid.xyz".toWebSocketUrl())
    }
}
