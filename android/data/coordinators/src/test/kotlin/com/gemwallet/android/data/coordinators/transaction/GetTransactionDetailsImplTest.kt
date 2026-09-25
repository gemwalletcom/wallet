package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStore
import com.gemwallet.android.model.Session
import com.gemwallet.android.serializer.jsonEncoder
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockGemTransactionDetailRows
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionExtended
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionExtended
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionSwapMetadata
import com.wallet.core.primitives.TransactionType
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemTransactionDetailsService
import uniffi.gemstone.BlockExplorerLink as GemBlockExplorerLink
import uniffi.gemstone.WalletType as GemWalletType

class GetTransactionDetailsImplTest {

    private val getSession = mockk<GetSession>()
    private val transactionStore = mockk<GemstoneTransactionStore>()
    private val transactionDetailsService = mockk<GemTransactionDetailsService>()

    private val subject = GetTransactionDetailsImpl(
        getSession = getSession,
        transactionStore = transactionStore,
        transactionDetailsService = transactionDetailsService,
    )

    @Test
    fun getTransactionDetails_buildsTheAggregateFromTheCoreRows() = runTest {
        val asset = mockAsset(id = mockAssetId(chain = Chain.Near))
        val transaction = mockTransaction(
            assetId = asset.id,
            from = "sender.near",
            to = "recipient.near",
            type = TransactionType.Swap,
            state = TransactionState.Confirmed,
            direction = TransactionDirection.Outgoing,
            feeAssetId = asset.id,
            metadata = jsonEncoder.encodeToString(
                TransactionSwapMetadata.serializer(),
                TransactionSwapMetadata(
                    fromAsset = asset.id,
                    toAsset = asset.id,
                    fromValue = "1",
                    toValue = "2",
                    provider = null,
                ),
            ),
        )
        val transactionExtended = mockTransactionExtended(
            transaction = transaction,
            asset = asset,
            feeAsset = asset,
            assets = listOf(asset),
        )
        val wallet = mockWallet(
            accounts = listOf(mockAccount(chain = Chain.Near, address = transaction.from)),
        )

        every { getSession() } returns MutableStateFlow(mockSession(wallet = wallet))
        every { transactionStore.observeTransaction(wallet.id, transaction.id) } returns flowOf(transactionExtended)
        every { transactionDetailsService.detailRows(any(), any()) } returns mockGemTransactionDetailRows(
            explorer = GemBlockExplorerLink("NEAR Intents", "https://explorer.near-intents.org/transactions/${transaction.to}"),
        )

        val result = subject.getTransactionDetails(transaction.id).first()

        assertNotNull(result)
        verify { transactionDetailsService.detailRows(any(), GemWalletType.MULTICOIN) }
        assertEquals("NEAR Intents", result?.rows?.explorer?.name)
        assertEquals(
            "https://explorer.near-intents.org/transactions/${transaction.to}",
            result?.rows?.explorer?.link,
        )
    }

    @Test
    fun getTransactionDetails_clearsWhenTheOtherWalletHasNoSuchRecord() = runTest {
        val transactionExtended = mockTransactionExtended()
        val transactionId = transactionExtended.transaction.id
        val first = mockWallet(id = "first")
        val second = mockWallet(id = "second")
        val session = MutableStateFlow<Session?>(mockSession(wallet = first))
        every { getSession() } returns session
        every { transactionStore.observeTransaction(first.id, transactionId) } returns flowOf(transactionExtended)
        every { transactionStore.observeTransaction(second.id, transactionId) } returns flowOf(null)
        every { transactionDetailsService.detailRows(any(), any()) } returns mockGemTransactionDetailRows()

        val details = subject.getTransactionDetails(transactionId).stateIn(backgroundScope)
        assertNotNull(details.value)

        session.value = mockSession(wallet = second)

        assertNull(details.first { it == null })
    }

    @Test
    fun getTransactionDetails_clearsWhenTheShownRecordIsDeleted() = runTest {
        val transactionExtended = mockTransactionExtended()
        val wallet = mockWallet()
        val record = MutableStateFlow<TransactionExtended?>(transactionExtended)
        every { getSession() } returns MutableStateFlow(mockSession(wallet = wallet))
        every { transactionStore.observeTransaction(wallet.id, transactionExtended.transaction.id) } returns record
        every { transactionDetailsService.detailRows(any(), any()) } returns mockGemTransactionDetailRows()

        val details = subject.getTransactionDetails(transactionExtended.transaction.id).stateIn(backgroundScope)
        assertNotNull(details.value)

        record.value = null

        assertNull(details.first { it == null })
    }
}
