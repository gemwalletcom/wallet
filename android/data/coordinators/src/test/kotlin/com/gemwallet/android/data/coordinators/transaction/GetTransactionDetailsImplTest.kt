package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.ext.hash
import uniffi.gemstone.GemBlockExplorerLink
import uniffi.gemstone.GemTransactionDetailsService
import uniffi.gemstone.GemTransactionHeaderKind
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransaction
import com.gemwallet.android.serializer.jsonEncoder
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionExtended
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionSwapMetadata
import com.wallet.core.primitives.TransactionType
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import com.gemwallet.android.testkit.mockGemTransactionDetails
import org.junit.Test

class GetTransactionDetailsImplTest {

    private val getSession = mockk<GetSession>()
    private val getTransaction = mockk<GetTransaction>()
    private val getWalletAssets = mockk<GetWalletAssets>()
    private val transactionDetailsService = mockk<GemTransactionDetailsService>()

    private val subject = GetTransactionDetailsImpl(
        getSession = getSession,
        getTransaction = getTransaction,
        getWalletAssets = getWalletAssets,
        transactionDetailsService = transactionDetailsService,
    )

    @Test
    fun getTransactionDetails_keepsSwapExplorerForTransactionWithoutCrashing() = runTest {
        val asset = mockAsset(chain = Chain.Near)
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
        every { getTransaction(transaction.id) } returns flowOf(transactionExtended)
        every { getWalletAssets(any<List<AssetId>>()) } returns flowOf(
            listOf(mockAssetInfo(asset = asset, owner = mockAccount(chain = Chain.Near, address = transaction.from)))
        )
        every { transactionDetailsService.transactionLink(Chain.Near.string, transaction.hash, any(), any(), any()) } returns GemBlockExplorerLink(
            "NEAR Intents",
            "https://explorer.near-intents.org/transactions/${transaction.to}",
        )
        every { transactionDetailsService.participant(any()) } returns null
        every { transactionDetailsService.headerKind(any()) } returns GemTransactionHeaderKind.Swap
        every { transactionDetailsService.details(any()) } returns mockGemTransactionDetails()

        val result = subject.getTransactionDetails(transaction.id).first()

        assertNotNull(result)
        assertEquals("NEAR Intents", result?.explorer?.name)
        assertEquals(
            "https://explorer.near-intents.org/transactions/${transaction.to}",
            result?.explorer?.url,
        )
    }
}
