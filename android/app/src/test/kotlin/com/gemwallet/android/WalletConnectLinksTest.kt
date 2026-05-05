package com.gemwallet.android

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class WalletConnectLinksTest {

    @Test
    fun walletConnectLink_acceptsDirectPairingUri() {
        val uri = "wc:abc@2?relay-protocol=irn&symKey=123"

        assertEquals(WalletConnectLink.Pairing(uri), uri.toWalletConnectLink())
    }

    @Test
    fun walletConnectLink_acceptsDirectCallbackUri() {
        assertEquals(WalletConnectLink.Request, "wc:abc@2?requestId".toWalletConnectLink())
        assertEquals(WalletConnectLink.Request, "wc:abc@2?requestId=123".toWalletConnectLink())
    }

    @Test
    fun walletConnectLink_acceptsGemPairingUri() {
        assertEquals(
            WalletConnectLink.Pairing("wc:abc@2?relay-protocol=irn&symKey=123"),
            "gem://wc?uri=wc%3Aabc%402%3Frelay-protocol%3Dirn%26symKey%3D123".toWalletConnectLink(),
        )
    }

    @Test
    fun walletConnectLink_acceptsGemCallbacks() {
        assertEquals(WalletConnectLink.Request, "gem://wc?requestId".toWalletConnectLink())
        assertEquals(WalletConnectLink.Request, "gem://wc?requestId=123".toWalletConnectLink())
        assertEquals(WalletConnectLink.Session, "gem://wc?sessionTopic=topic".toWalletConnectLink())
    }

    @Test
    fun walletConnectLink_rejectsEmptySessionCallback() {
        assertNull("gem://wc?sessionTopic=".toWalletConnectLink())
    }

    @Test
    fun walletConnectLink_rejectsNormalNavigationLinks() {
        assertNull("gem://asset/solana".toWalletConnectLink())
        assertNull("https://gemwallet.com/join?code=abc123".toWalletConnectLink())
    }
}
