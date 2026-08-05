package com.gemwallet.android.blockchain.services

import io.mockk.coEvery
import io.mockk.mockk
import io.mockk.slot
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.SignDigestType
import uniffi.gemstone.SignMessage
import uniffi.gemstone.SimulationResult
import uniffi.gemstone.WalletConnectSimulationClientInterface

private const val TYPED_DATA = """{"domain":{"name":"Permit2"},"message":{"spender":"0xspender"}}"""

class WalletConnectSimulationServiceTest {

    private val client = mockk<WalletConnectSimulationClientInterface>()
    private val service = WalletConnectSimulationService(client)

    @Test
    fun simulateSignMessage_sendsTypedDataAsTextTheSimulatorCanParse() = runBlocking {
        val sent = slot<String>()
        coEvery { client.simulateSignMessage(any(), any(), capture(sent), any()) } returns SimulationResult(
            warnings = emptyList(),
            balanceChanges = emptyList(),
            payload = emptyList(),
            header = null,
        )

        service.simulateSignMessage(
            SignMessage(chain = "ethereum", signType = SignDigestType.EIP712, data = TYPED_DATA.toByteArray()),
            sessionDomain = "https://pay.walletconnect.com",
        )

        assertEquals(TYPED_DATA, sent.captured)
    }
}
