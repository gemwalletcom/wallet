package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.cases.stake.SyncStakeDelegations
import com.gemwallet.android.ext.getAssociatedAssetIds
import com.gemwallet.android.ext.isCompleted
import com.gemwallet.android.model.TransactionExtended
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class TransactionPostProcessingService @Inject constructor(
    private val assetsRepository: AssetsRepository,
    private val syncStakeDelegations: SyncStakeDelegations,
    private val syncNfts: SyncNfts,
) {

    internal suspend fun processTransactions(transactions: List<TransactionExtended>) = withContext(Dispatchers.IO) {
        transactions.map { transactionExtended ->
            async {
                val transaction = transactionExtended.transaction
                val assetInfos = assetsRepository.getAssetsInfo(transaction.getAssociatedAssetIds()).firstOrNull().orEmpty()
                assetsRepository.updateBalances(assetInfos)
                val walletId = assetInfos.firstNotNullOfOrNull { it.walletId } ?: return@async
                if (transaction.state.isCompleted()) {
                    processCompleteTransaction(walletId, transaction)
                }
            }
        }.awaitAll()
    }

    private suspend fun processCompleteTransaction(
        walletId: WalletId,
        transaction: Transaction,
    ) {
        when (transaction.type) {
            TransactionType.StakeDelegate,
            TransactionType.StakeUndelegate,
            TransactionType.StakeRewards,
            TransactionType.StakeRedelegate,
            TransactionType.StakeWithdraw,
            TransactionType.StakeFreeze,
            TransactionType.StakeUnfreeze -> syncStakeDelegations.sync(
                walletId = walletId,
                assetId = transaction.assetId,
                address = transaction.from,
            )
            TransactionType.TransferNFT -> syncNfts.sync(walletId)
            else -> Unit
        }
    }
}
