package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.model.ServiceUnavailable
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import io.mockk.coEvery
import io.mockk.mockk
import io.mockk.slot
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test
import uniffi.gemstone.GemGatewayInterface
import uniffi.gemstone.TransactionChange
import uniffi.gemstone.TransactionUpdate

class TransactionStatusServiceTest {

    private val gateway = mockk<GemGatewayInterface>()
    private val service = TransactionStatusService(gateway)

    private fun transaction() = Transaction(
        id = TransactionId(Chain.Bitcoin, "0xhash"),
        assetId = AssetId(Chain.Bitcoin),
        from = "sender",
        to = "recipient",
        contract = null,
        type = TransactionType.Transfer,
        state = TransactionState.Pending,
        blockNumber = "10",
        sequence = "0",
        fee = "1",
        feeAssetId = AssetId(Chain.Bitcoin),
        value = "100",
        memo = null,
        direction = TransactionDirection.Outgoing,
        utxoInputs = null,
        utxoOutputs = null,
        metadata = null,
        createdAt = 1234L,
    )

    @Test
    fun getUpdate_sendsTheTransactionAndMapsTheResult() = runBlocking {
        val sent = slot<String>()
        coEvery { gateway.getTransactionUpdate(capture(sent)) } returns TransactionUpdate(
            state = TransactionState.Confirmed.toJson(),
            changes = listOf(TransactionChange.NetworkFee("42")),
        )

        val result = service.getUpdate(transaction())

        assertEquals("0xhash", sent.captured.decodeJson<Transaction>().id.hash)
        assertEquals(TransactionState.Confirmed, result.state)
        assertEquals("42", result.fee?.toString())
    }

    @Test
    fun getUpdate_reportsGatewayFailureAsServiceUnavailable() = runBlocking {
        coEvery { gateway.getTransactionUpdate(any()) } throws IllegalStateException("boom")

        assertThrows(ServiceUnavailable::class.java) {
            runBlocking { service.getUpdate(transaction()) }
        }
        Unit
    }
}
