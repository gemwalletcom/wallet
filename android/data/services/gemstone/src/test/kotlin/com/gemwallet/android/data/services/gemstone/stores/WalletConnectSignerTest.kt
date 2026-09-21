package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequests
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockApplicationMetadata
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockGemWalletConnectMessageRequest
import com.gemwallet.android.testkit.mockGemWalletConnectTransactionRequest
import com.gemwallet.android.testkit.mockSignMessage
import com.gemwallet.android.testkit.mockTransferDataExtra
import com.gemwallet.android.testkit.mockWalletConnectionSession
import com.gemwallet.android.testkit.mockWalletMulticoin
import com.wallet.core.primitives.TransactionType
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemWalletConnectTransactionAction
import uniffi.gemstone.TransactionInputType

class WalletConnectSignerTest {
    private val wallet = mockWalletMulticoin()
    private val session = mockWalletConnectionSession(id = "topic", metadata = mockApplicationMetadata(name = "dapp"))
    private val pendingRequests = WalletConnectPendingRequests()

    @Test
    fun `sign message waits for the approved pending request`() = runTest {
        val request = mockGemWalletConnectMessageRequest(wallet = wallet, session = session, message = mockSignMessage(data = "hello".toByteArray()))
        val result = async { pendingRequests.signMessage(request) }
        val pending = pendingRequests.current.filterNotNull().first()
        assertEquals(wallet.id, pending.wallet.id)
        assertEquals("dapp", pending.appMetadata.name)
        pending.approve("0xsig")
        assertEquals("0xsig", result.await())
        assertNull(pendingRequests.current.value)
    }

    @Test
    fun `send transaction is marked sendable`() = runTest {
        val transfer = mockGemTransferData(
            inputType = TransactionInputType.Generic(
                asset = mockAssetSolana().toGem(),
                metadata = mockApplicationMetadata().toGem(),
                extra = mockTransferDataExtra(data = "tx".toByteArray(), transactionType = TransactionType.SmartContractCall),
            ),
        )
        val request = mockGemWalletConnectTransactionRequest(transfer, GemWalletConnectTransactionAction.SEND, wallet = wallet, session = session)
        val result = async { pendingRequests.signTransaction(request) }
        val pending = pendingRequests.current.filterNotNull().first() as WalletConnectPendingRequest.Transaction
        assertTrue(pending.isSendable)
        pending.approve("hash")
        assertEquals("hash", result.await())
    }
}
