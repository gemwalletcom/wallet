package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.transactions.TransactionRepository
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
import com.wallet.core.primitives.SwapProvider
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionSwapMetadata
import com.wallet.core.primitives.TransactionType
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test
import uniffi.gemstone.GemSwapper

class GetTransactionDetailsImplTest {

    private val sessionRepository = mockk<SessionRepository>()
    private val transactionRepository = mockk<TransactionRepository>()
    private val assetsRepository = mockk<AssetsRepository>()
    private val getCurrentBlockExplorer = mockk<GetCurrentBlockExplorer>()
    private val gemSwapper = mockk<GemSwapper>()
    private val explorer = mockk<uniffi.gemstone.Explorer>()

    private val subject = GetTransactionDetailsImpl(
        sessionRepository = sessionRepository,
        transactionRepository = transactionRepository,
        assetsRepository = assetsRepository,
        getCurrentBlockExplorer = getCurrentBlockExplorer,
        gemSwapper = gemSwapper,
        createExplorer = { explorer },
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
                    provider = SwapProvider.NearIntents.string,
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

        every { sessionRepository.session() } returns MutableStateFlow(mockSession(wallet = wallet))
        every { transactionRepository.getTransaction("tx-id") } returns flowOf(transactionExtended)
        every { assetsRepository.getAssetsInfo(any<List<AssetId>>()) } returns flowOf(
            listOf(mockAssetInfo(asset = asset, owner = mockAccount(chain = Chain.Near, address = transaction.from)))
        )
        every { getCurrentBlockExplorer.getBlockExplorerInfo(transaction) } returns Pair(
            "https://explorer.near-intents.org/transactions/${transaction.to}",
            "NEAR Intents",
        )
        every { getCurrentBlockExplorer.getCurrentBlockExplorer(Chain.Near) } returns "Near"
        every { gemSwapper.getProviders() } returns emptyList()
        every { explorer.getAddressUrl("Near", any()) } answers { "https://nearblocks.io/address/${secondArg<String>()}" }

        val result = subject.getTransactionDetails("tx-id").first()

        assertNotNull(result)
        assertEquals("NEAR Intents", result?.explorer?.name)
        assertEquals(
            "https://explorer.near-intents.org/transactions/${transaction.to}",
            result?.explorer?.url,
        )
        verify(exactly = 2) { explorer.getAddressUrl("Near", any()) }
        verify(exactly = 0) { explorer.getAddressUrl("NEAR Intents", any()) }
    }
}
