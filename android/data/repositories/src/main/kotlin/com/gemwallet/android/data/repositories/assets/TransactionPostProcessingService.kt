package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.application.transactions.coordinators.GetChangedTransactions
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.cases.stake.SyncStakeDelegations
import com.gemwallet.android.ext.getAssociatedAssetIds
import com.gemwallet.android.ext.isCompleted
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.TransactionExtended
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class TransactionPostProcessingService(
    getChangedTransactions: GetChangedTransactions,
    private val assetsRepository: AssetsRepository,
    private val syncStakeDelegations: SyncStakeDelegations,
    private val syncNfts: SyncNfts,
    private val scope: CoroutineScope,
) {

    @Inject
    constructor(
        getChangedTransactions: GetChangedTransactions,
        assetsRepository: AssetsRepository,
        syncStakeDelegations: SyncStakeDelegations,
        syncNfts: SyncNfts,
    ) : this(getChangedTransactions, assetsRepository, syncStakeDelegations, syncNfts, CoroutineScope(Dispatchers.IO))

    init {
        scope.launch(Dispatchers.IO) {
            getChangedTransactions.getChangedTransactions().collect {
                processTransactions(it)
            }
        }
    }

    internal suspend fun processTransactions(transactions: List<TransactionExtended>) = withContext(Dispatchers.IO) {
        transactions.map { transactionExtended ->
            async {
                val transaction = transactionExtended.transaction
                val assetInfos = assetsRepository.getAssetsInfo(transaction.getAssociatedAssetIds()).firstOrNull().orEmpty()
                assetsRepository.updateBalances(assetInfos)
                val walletId = assetInfos.firstNotNullOfOrNull { it.walletId } ?: return@async
                if (transaction.state.isCompleted()) {
                    processCompleteTransaction(walletId, transaction, assetInfos)
                }
            }
        }.awaitAll()
    }

    private suspend fun processCompleteTransaction(
        walletId: WalletId,
        transaction: Transaction,
        assetInfos: List<AssetInfo>,
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
                apr = assetInfos.firstOrNull { it.id() == transaction.assetId }?.stakeApr ?: 0.0,
            )
            TransactionType.TransferNFT -> syncNfts.sync(walletId)
            else -> Unit
        }
    }
}
