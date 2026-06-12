package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.application.transactions.coordinators.GetChangedTransactions
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.cases.stake.SyncStakeDelegations
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionExtended
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Test

class TransactionPostProcessingServiceTest {

    private val getChangedTransactions = mockk<GetChangedTransactions>()
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)
    private val syncStakeDelegations = mockk<SyncStakeDelegations>(relaxed = true)
    private val syncNfts = mockk<SyncNfts>(relaxed = true)
    private val scope = CoroutineScope(Job())

    private fun createSubject() = TransactionPostProcessingService(
        getChangedTransactions = getChangedTransactions,
        assetsRepository = assetsRepository,
        syncStakeDelegations = syncStakeDelegations,
        syncNfts = syncNfts,
        scope = scope,
    )

    @After
    fun tearDown() {
        scope.cancel()
        unmockkAll()
    }

    @Test
    fun completeStakeTransaction_syncsDelegations() = runBlocking {
        every { getChangedTransactions.getChangedTransactions() } returns emptyFlow()
        val asset = mockAssetSolana()
        val assetInfo = mockAssetInfo(asset = asset, walletId = mockWalletId("wallet-1")).copy(stakeApr = 7.5)
        every { assetsRepository.getAssetsInfo(any<List<AssetId>>()) } returns flowOf(listOf(assetInfo))

        val subject = createSubject()
        subject.processTransactions(
            listOf(TransactionState.Confirmed, TransactionState.Failed, TransactionState.Reverted).map { state ->
                mockTransactionExtended(
                    transaction = mockTransaction(
                        assetId = asset.id,
                        from = "solana-sender",
                        type = TransactionType.StakeDelegate,
                        state = state,
                    ),
                    asset = asset,
                )
            }
        )

        coVerify(exactly = 3) {
            syncStakeDelegations.sync(mockWalletId("wallet-1"), asset.id, "solana-sender", apr = 7.5)
        }
        coVerify(exactly = 0) { syncNfts.sync(any()) }
    }

    @Test
    fun completeNftTransfer_syncsWalletNfts() = runBlocking {
        every { getChangedTransactions.getChangedTransactions() } returns emptyFlow()
        val transaction = mockTransaction(type = TransactionType.TransferNFT)
        val assetInfo: AssetInfo = mockAssetInfo(walletId = mockWalletId())
        every { assetsRepository.getAssetsInfo(any<List<AssetId>>()) } returns flowOf(listOf(assetInfo))

        val subject = createSubject()
        subject.processTransactions(
            listOf(TransactionState.Confirmed, TransactionState.Failed, TransactionState.Reverted).map { state ->
                mockTransactionExtended(transaction = transaction.copy(state = state))
            }
        )

        coVerify(exactly = 3) { syncNfts.sync(mockWalletId()) }
        coVerify(exactly = 0) { syncStakeDelegations.sync(any(), any(), any(), any()) }
    }
}
