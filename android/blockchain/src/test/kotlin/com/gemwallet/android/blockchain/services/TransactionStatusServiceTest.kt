package com.gemwallet.android.blockchain.services

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SwapProvider
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionStateRequest
import com.wallet.core.primitives.TransactionSwapStateRequest
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import io.mockk.slot
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemGatewayInterface
import uniffi.gemstone.GemTransactionStateRequest
import uniffi.gemstone.GemTransactionSwapStateRequest
import uniffi.gemstone.SwapperProvider
import uniffi.gemstone.SwapperTransactionSwapMetadata
import uniffi.gemstone.TransactionChange
import uniffi.gemstone.TransactionMetadata
import uniffi.gemstone.TransactionUpdate

class TransactionStatusServiceTest {

    private val gateway = mockk<GemGatewayInterface>()
    private val service = TransactionStatusService(gateway)

    @Test
    fun getStatus_mapsRequest() = runBlocking {
        val requestSlot = slot<GemTransactionStateRequest>()
        coEvery { gateway.getTransactionStatus(Chain.Bitcoin.string, capture(requestSlot)) } returns TransactionUpdate(
            state = uniffi.gemstone.TransactionState.CONFIRMED,
            changes = emptyList(),
        )

        val result = service.getStatus(
            Chain.Bitcoin,
            TransactionStateRequest(
                id = "0xhash",
                senderAddress = "sender",
                createdAt = 1_234_000L,
                blockNumber = 10L,
            )
        )

        assertEquals(TransactionState.Confirmed, result.state)
        assertEquals("0xhash", requestSlot.captured.id)
        assertEquals("sender", requestSlot.captured.senderAddress)
        assertEquals(1234L, requestSlot.captured.createdAt)
        assertEquals(10UL, requestSlot.captured.blockNumber)
        coVerify(exactly = 1) { gateway.getTransactionStatus(Chain.Bitcoin.string, any()) }
    }

    @Test
    fun getSwapStatus_mapsInTransitMetadataAndRequest() = runBlocking {
        val requestSlot = slot<GemTransactionSwapStateRequest>()
        coEvery { gateway.getTransactionSwapStatus(Chain.Bitcoin.string, capture(requestSlot)) } returns TransactionUpdate(
            state = uniffi.gemstone.TransactionState.IN_TRANSIT,
            changes = listOf(
                TransactionChange.Metadata(
                    TransactionMetadata.Swap(
                        SwapperTransactionSwapMetadata(
                            fromAsset = "bitcoin",
                            fromValue = "100000000",
                            toAsset = "ethereum",
                            toValue = "9900000000000000000",
                            provider = "thorchain",
                        )
                    )
                )
            ),
        )

        val result = service.getSwapStatus(
            Chain.Bitcoin,
            TransactionSwapStateRequest(
                transaction = TransactionStateRequest(
                    id = "0xhash",
                    senderAddress = "sender",
                    createdAt = 1_234_000L,
                    blockNumber = 10L,
                ),
                state = TransactionState.InTransit,
                swapProvider = SwapProvider.Thorchain,
                destinationChain = Chain.Ethereum,
            )
        )

        assertEquals(TransactionState.InTransit, result.state)
        assertEquals(
            "{\"fromAsset\":\"bitcoin\",\"fromValue\":\"100000000\",\"toAsset\":\"ethereum\",\"toValue\":\"9900000000000000000\",\"provider\":\"thorchain\"}",
            result.metadata,
        )
        assertEquals("0xhash", requestSlot.captured.transaction.id)
        assertEquals("sender", requestSlot.captured.transaction.senderAddress)
        assertEquals(1234L, requestSlot.captured.transaction.createdAt)
        assertEquals(10UL, requestSlot.captured.transaction.blockNumber)
        assertEquals(uniffi.gemstone.TransactionState.IN_TRANSIT, requestSlot.captured.state)
        assertEquals(SwapperProvider.THORCHAIN, requestSlot.captured.swapProvider)
        assertEquals(Chain.Ethereum.string, requestSlot.captured.destinationChain)
        coVerify(exactly = 1) { gateway.getTransactionSwapStatus(Chain.Bitcoin.string, any()) }
    }
}
