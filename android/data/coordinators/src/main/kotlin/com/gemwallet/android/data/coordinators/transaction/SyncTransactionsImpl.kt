package com.gemwallet.android.data.coordinators.transaction

import android.content.Context
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.service.store.WalletPreferences
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.ext.getAssociatedAssetIds
import com.gemwallet.android.ext.identifier
import com.gemwallet.android.model.Transaction
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet

class SyncTransactionsImpl(
    private val context: Context,
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val saveTransactions: SaveTransactions,
    private val assetsRepository: AssetsRepository,
) : SyncTransactions {

    override suspend fun syncTransactions(wallet: Wallet) {
        val preferences = WalletPreferences(context, wallet.id)
        val transactions = runCatching {
            gemDeviceApiClient.getTransactions(wallet.id, preferences.transactionsTimestamp)?.transactions
        }.getOrNull() ?: return

        prefetchAssets(wallet, transactions)
        saveTransactions.saveTransactions(walletId = wallet.id, transactions)
        preferences.transactionsTimestamp = currentTimestamp()
    }

    override suspend fun syncTransactions(wallet: Wallet, assetId: AssetId) {
        val preferences = WalletPreferences(context, wallet.id)
        val assetId = assetId.identifier
        val timestamp = preferences.transactionsForAssetTimestamp(assetId)
        val transactions = runCatching {
            gemDeviceApiClient.getTransactions(wallet.id, assetId, timestamp)?.transactions
        }.getOrNull() ?: return

        prefetchAssets(wallet, transactions)
        saveTransactions.saveTransactions(walletId = wallet.id, transactions)
        preferences.setTransactionsForAssetTimestamp(assetId, currentTimestamp())
    }

    private fun currentTimestamp(): Long = System.currentTimeMillis() / 1000

    private suspend fun prefetchAssets(wallet: Wallet, transactions: List<Transaction>) {
        val assetIds = transactions.map { transaction ->
            transaction.getAssociatedAssetIds().filter { assetsRepository.getAsset(it) == null }.toSet()
        }.flatten()
        assetsRepository.resolve(wallet, assetIds)
    }
}
