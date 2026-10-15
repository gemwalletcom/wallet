package com.gemwallet.android.application.wallet_connect

import org.junit.Assert.assertEquals
import org.junit.Test

class SessionTest {

    @Test
    fun addingAccountExtendsItsNamespace() {
        val ethereum = WalletConnectSessionNamespace(chains = listOf("eip155:1"), methods = listOf("eth_sendTransaction"), events = listOf("chainChanged"), accounts = listOf("eip155:1:0xabc"))
        val solana = WalletConnectSessionNamespace(chains = null, methods = listOf("solana_signMessage"), events = emptyList(), accounts = listOf("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp:address"))
        val namespaces = mapOf("eip155" to ethereum, "solana" to solana)

        val updated = namespaces.addingAccount("eip155:81457:0xabc")

        assertEquals(ethereum.copy(chains = listOf("eip155:1", "eip155:81457"), accounts = listOf("eip155:1:0xabc", "eip155:81457:0xabc")), updated["eip155"])
        assertEquals(solana, updated["solana"])
    }
}
