package com.gemwallet.android.data.repositories.transactions

import com.gemwallet.android.cases.transactions.GetTransactionUpdateTime
import com.gemwallet.android.cases.transactions.PutTransactions
import com.gemwallet.android.cases.transactions.SyncTransactions
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.ext.getAssociatedAssetIds
import com.gemwallet.android.ext.identifier
import com.gemwallet.android.model.Transaction
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import javax.inject.Inject

class SyncTransactionsService @Inject constructor(
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val putTransactions: PutTransactions,
    private val getTransactionUpdateTime: GetTransactionUpdateTime,
    private val assetsRepository: AssetsRepository,
) : SyncTransactions {

    override suspend fun syncTransactions(wallet: Wallet) {
        val lastSyncTime = getTransactionUpdateTime.getTransactionUpdateTime(wallet.id) / 1000L
        val response = runCatching {
            val result: List<Transaction>? = try {
                gemDeviceApiClient.getTransactions(wallet.id, lastSyncTime)?.transactions
            } catch (_: Throwable) {
                null
            }
            result
        }
        val transactions: List<Transaction> = response.getOrNull() ?: return
        prefetchAssets(wallet, transactions)

        putTransactions.putTransactions(walletId = wallet.id, transactions.toList())
    }

    override suspend fun syncTransactions(wallet: Wallet, assetId: AssetId) = withContext(Dispatchers.IO) {
        val lastSyncTime = getTransactionUpdateTime.getTransactionUpdateTime(wallet.id, assetId.identifier) / 1000L
        val response = runCatching {
            val result: List<Transaction>? = try {
                gemDeviceApiClient.getTransactions(wallet.id, assetId.identifier, lastSyncTime)?.transactions
            } catch (_: Throwable) {
                null
            }
            result
        }
        val transactions: List<Transaction> = response.getOrNull() ?: return@withContext
        prefetchAssets(wallet, transactions)

        putTransactions.putTransactions(walletId = wallet.id, transactions.toList())
    }

    private suspend fun prefetchAssets(wallet: Wallet, transactions: List<Transaction>) {
        val notAvailableAssetIds = transactions.map { transaction ->
            transaction.getAssociatedAssetIds().filter { assetsRepository.getAsset(it) == null }.toSet()
        }.flatten()
        assetsRepository.resolve(wallet, notAvailableAssetIds)
    }
}