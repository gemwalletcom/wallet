package com.gemwallet.android.data.services.gemstone.stores

import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.TransferDataOutputAction
import com.wallet.core.primitives.TransferDataOutputType
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequests
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
import uniffi.gemstone.GemWalletConnectMessageRequest
import uniffi.gemstone.GemWalletConnectTransactionRequest
import uniffi.gemstone.GemWalletConnectTransactionAction
import uniffi.gemstone.SignDigestType
import uniffi.gemstone.SignMessage
import com.gemwallet.android.ext.asset
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.TransferDataExtra
import java.math.BigInteger

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
    private val simulation = SimulationResult(warnings = emptyList(), balanceChanges = emptyList(), payload = emptyList()).toJson()

    private fun messageRequest(message: SignMessage) = GemWalletConnectMessageRequest(
        sessionId = "topic",
        chain = Chain.Ethereum.string,
        wallet = wallet.toGem(),
        account = account.toGem(),
        session = session.toGem(),
        simulation = simulation,
        message = message,
    )

    private fun transactionRequest(transfer: GemTransferData, action: GemWalletConnectTransactionAction) = GemWalletConnectTransactionRequest(
        sessionId = "topic",
        chain = Chain.Ethereum.string,
        wallet = wallet.toGem(),
        account = account.toGem(),
        session = session.toGem(),
        simulation = simulation,
        transfer = transfer,
        action = action,
    )

    @Test
    fun `sign message waits for the approved pending request`() = runTest {
        val message = SignMessage(chain = Chain.Ethereum.string, signType = SignDigestType.EIP191, data = "hello".toByteArray())
        val result = async { pendingRequests.signMessage(messageRequest(message)) }
        val pending = pendingRequests.current.filterNotNull().first()
        assertEquals(wallet.id, pending.wallet.id)
        assertEquals("dapp", pending.appMetadata.name)
        pending.approve("0xsig")
        assertEquals("0xsig", result.await())
        assertNull(pendingRequests.current.value)
    }

    @Test
    fun `send transaction is marked sendable`() = runTest {
        val transfer = GemTransferData(
            inputType = GemTransactionInputType.Generic(
                asset = Chain.Solana.asset().toGem(),
                metadata = ApplicationMetadata(name = "dapp", description = "", url = "https://dapp.example", icon = "", source = ApplicationMetadataSource.WalletConnect).toGem(),
                extra = TransferDataExtra(
                    to = "",
                    gasLimit = null,
                    gasPrice = null,
                    data = "tx".toByteArray(),
                    outputType = TransferDataOutputType.EncodedTransaction.toGem(),
                    outputAction = TransferDataOutputAction.Send.toGem(),
                    transactionType = TransactionType.SmartContractCall.toGem(),
                    approval = null,
                ),
            ),
            recipient = GemRecipient(address = "", name = null, memo = null, references = emptyList()),
            value = BigInteger.ZERO,
            useMaxAmount = false,
            minimumValue = null,
        )
        val result = async { pendingRequests.signTransaction(transactionRequest(transfer, GemWalletConnectTransactionAction.SEND)) }
        val pending = pendingRequests.current.filterNotNull().first() as WalletConnectPendingRequest.Transaction
        assertTrue(pending.isSendable)
        pending.approve("hash")
        assertEquals("hash", result.await())
    }
}
