package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.repositories.bridge.ConnectionsRepository
import com.gemwallet.android.data.repositories.bridge.WalletConnectPendingRequests
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.ApplicationMetadataSource
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationResult
import com.wallet.core.primitives.WalletConnection
import com.wallet.core.primitives.WalletConnectionSession
import com.wallet.core.primitives.WalletConnectionState
import io.mockk.coEvery
import io.mockk.mockk
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertThrows
import org.junit.Test
import uniffi.gemstone.SignDigestType
import uniffi.gemstone.SignMessage

class WalletConnectSignerTest {
    private val wallet = mockWallet(accounts = listOf(mockAccount(chain = Chain.Ethereum)))
    private val connection = WalletConnection(
        session = WalletConnectionSession(
            id = "topic",
            sessionId = "session",
            state = WalletConnectionState.Active,
            chains = listOf(Chain.Ethereum),
            createdAt = 0,
            expireAt = 0,
            metadata = ApplicationMetadata(name = "dapp", description = "", url = "https://dapp", icon = "", source = ApplicationMetadataSource.WalletConnect),
        ),
        wallet = wallet,
    )
    private val connections = mockk<ConnectionsRepository> {
        coEvery { getConnectionByTopic("topic") } returns connection
    }
    private val pendingRequests = WalletConnectPendingRequests()
    private val signer = GemstoneWalletConnectSigner(connections, pendingRequests)
    private val simulation = SimulationResult(warnings = emptyList(), balanceChanges = emptyList(), payload = emptyList()).toJson()
    private val message = SignMessage(chain = Chain.Ethereum.string, signType = SignDigestType.EIP191, data = "hello".toByteArray())

    @Test
    fun `sign message waits for the approved pending request`() = runTest {
        val result = async { signer.signMessage("topic", Chain.Ethereum.string, message, simulation) }

        val pending = pendingRequests.current.filterNotNull().first()
        assertEquals(wallet.id, pending.wallet.id)
        pending.approve("0xsig")

        assertEquals("0xsig", result.await())
        assertNull(pendingRequests.current.value)
    }

    @Test
    fun `chains outside the session are refused before asking the user`() = runTest {
        assertThrows(IllegalStateException::class.java) {
            kotlinx.coroutines.runBlocking { signer.getAccounts("topic", Chain.Solana.string) }
        }
        assertNull(pendingRequests.current.value)
    }
}
