package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.repositories.bridge.WalletConnectPendingRequest
import com.gemwallet.android.data.repositories.bridge.WalletConnectPendingRequests
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.ApplicationMetadataSource
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationResult
import com.wallet.core.primitives.WalletConnectionSession
import com.wallet.core.primitives.WalletConnectionState
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemWalletConnectSignPayload
import uniffi.gemstone.GemWalletConnectSignRequest
import uniffi.gemstone.GemWalletConnectTransactionAction
import uniffi.gemstone.SignDigestType
import uniffi.gemstone.SignMessage
import uniffi.gemstone.WcSolanaTransactionData
import uniffi.gemstone.WalletConnectTransaction

class WalletConnectSignerTest {
    private val account = mockAccount(chain = Chain.Ethereum)
    private val wallet = mockWallet(accounts = listOf(account))
    private val session = WalletConnectionSession(
        id = "topic",
        sessionId = "topic",
        state = WalletConnectionState.Active,
        chains = listOf(Chain.Ethereum),
        createdAt = 0,
        expireAt = 0,
        metadata = ApplicationMetadata(name = "dapp", description = "", url = "https://dapp", icon = "", source = ApplicationMetadataSource.WalletConnect),
    )
    private val pendingRequests = WalletConnectPendingRequests()
    private val signer = GemstoneWalletConnectSigner(pendingRequests)
    private val simulation = SimulationResult(warnings = emptyList(), balanceChanges = emptyList(), payload = emptyList()).toJson()

    private fun request(payload: GemWalletConnectSignPayload) = GemWalletConnectSignRequest(
        sessionId = "topic",
        chain = Chain.Ethereum.string,
        wallet = wallet.toJson(),
        account = account.toGem(),
        session = session.toJson(),
        simulation = simulation,
        payload = payload,
    )

    @Test
    fun `sign message waits for the approved pending request`() = runTest {
        val message = SignMessage(chain = Chain.Ethereum.string, signType = SignDigestType.EIP191, data = "hello".toByteArray())
        val result = async { signer.sign(request(GemWalletConnectSignPayload.Message(message))) }
        val pending = pendingRequests.current.filterNotNull().first()
        assertEquals(wallet.id, pending.wallet.id)
        assertEquals("dapp", pending.appMetadata.name)
        pending.approve("0xsig")
        assertEquals("0xsig", result.await())
        assertNull(pendingRequests.current.value)
    }

    @Test
    fun `send transaction is marked sendable`() = runTest {
        val transaction = WalletConnectTransaction.Solana(
            data = WcSolanaTransactionData(transaction = "tx"),
            outputType = "\"encodedTransaction\"",
            transactionType = "\"smartContractCall\"",
        )
        val result = async { signer.sign(request(GemWalletConnectSignPayload.Transaction(transaction, GemWalletConnectTransactionAction.SEND))) }
        val pending = pendingRequests.current.filterNotNull().first() as WalletConnectPendingRequest.Transaction
        assertTrue(pending.isSendable)
        pending.approve("hash")
        assertEquals("hash", result.await())
    }
}
