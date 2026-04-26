package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.assets.coordinators.EnsureWalletAssets
import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.cases.addresses.SaveAddressNames
import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.ext.getAssociatedAssetIds
import com.gemwallet.android.ext.identifier
import com.gemwallet.android.model.TransactionsResponse
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet

class SyncTransactionsImpl(
    private val walletPreferencesFactory: WalletPreferencesFactory,
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val saveTransactions: SaveTransactions,
    private val saveAddressNames: SaveAddressNames,
    private val prefetchAssets: PrefetchAssets,
    private val ensureWalletAssets: EnsureWalletAssets,
) : SyncTransactions {

    override suspend fun syncTransactions(wallet: Wallet) {
        val preferences = walletPreferencesFactory.create(wallet.id)
        val response = runCatching {
            gemDeviceApiClient.getTransactions(wallet.id, preferences.transactionsTimestamp)
        }.getOrNull() ?: return

        sync(wallet, response)
        preferences.transactionsTimestamp = currentTimestamp()
    }

    override suspend fun syncTransactions(wallet: Wallet, assetId: AssetId) {
        val preferences = walletPreferencesFactory.create(wallet.id)
        val assetId = assetId.identifier
        val timestamp = preferences.transactionsForAssetTimestamp(assetId)
        val response = runCatching {
            gemDeviceApiClient.getTransactions(wallet.id, assetId, timestamp)
        }.getOrNull() ?: return

        sync(wallet, response)
        preferences.setTransactionsForAssetTimestamp(assetId, currentTimestamp())
    }

    private suspend fun sync(wallet: Wallet, response: TransactionsResponse) {
        val assetIds = response.transactions
            .flatMap { it.getAssociatedAssetIds() }
            .distinct()
        prefetchAssets.prefetchAssets(assetIds)
        ensureWalletAssets.ensureWalletAssets(wallet, assetIds)

        saveTransactions.saveTransactions(walletId = wallet.id, response.transactions)
        saveAddressNames.saveAddressNames(response.addressNames)
    }

    private fun currentTimestamp(): Long = System.currentTimeMillis() / 1000
}
