package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.assets.coordinators.EnsureWalletAssets
import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.transactions.coordinators.SyncAssetTransactions
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.cases.addresses.SaveAddressNames
import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.ext.getAssociatedAssetIds
import com.gemwallet.android.ext.identifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.TransactionsResponse
import com.wallet.core.primitives.Wallet
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class SyncTransactionsImpl @Inject constructor(
    private val walletPreferencesFactory: WalletPreferencesFactory,
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val saveTransactions: SaveTransactions,
    private val saveAddressNames: SaveAddressNames,
    private val prefetchAssets: PrefetchAssets,
    private val ensureWalletAssets: EnsureWalletAssets,
    private val sessionRepository: SessionRepository,
) : SyncTransactions, SyncAssetTransactions {

    override suspend fun syncTransactions(wallet: Wallet) {
        val walletId = wallet.id.id
        val preferences = walletPreferencesFactory.create(walletId)
        val response = runCatching {
            gemDeviceApiClient.getTransactions(walletId, preferences.transactionsTimestamp)
        }.getOrNull() ?: return

        sync(wallet, response)
        preferences.transactionsTimestamp = currentTimestamp()
    }

    override suspend fun syncAssetTransactions(assetId: AssetId) {
        val wallet = sessionRepository.getCurrentWallet() ?: return

        syncAssetTransactions(wallet, assetId)
    }

    private suspend fun syncAssetTransactions(wallet: Wallet, assetId: AssetId) {
        val walletId = wallet.id.id
        val preferences = walletPreferencesFactory.create(walletId)
        val assetIdentifier = assetId.identifier
        val timestamp = preferences.transactionsForAssetTimestamp(assetIdentifier)
        val response = runCatching {
            gemDeviceApiClient.getTransactions(walletId, assetIdentifier, timestamp)
        }.getOrNull() ?: return

        sync(wallet, response)
        preferences.setTransactionsForAssetTimestamp(assetIdentifier, currentTimestamp())
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
