package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.transactions.coordinators.SyncAssetTransactions
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.cases.addresses.SaveAddressNames
import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.ext.currentTimestamp
import com.gemwallet.android.ext.getAssociatedAssetIds
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.TransactionsResponse
import com.wallet.core.primitives.Wallet
import javax.inject.Inject
import javax.inject.Singleton
import uniffi.gemstone.GemTransactionsService
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson

@Singleton
class SyncTransactionsImpl @Inject constructor(
    private val walletPreferencesFactory: WalletPreferencesFactory,
    private val transactionsService: GemTransactionsService,
    private val saveTransactions: SaveTransactions,
    private val saveAddressNames: SaveAddressNames,
    private val prefetchAssets: PrefetchAssets,
    private val assetsRepository: AssetsRepository,
    private val sessionRepository: SessionRepository,
) : SyncTransactions, SyncAssetTransactions {

    override suspend fun syncTransactions(wallet: Wallet) {
        val walletId = wallet.id
        val preferences = walletPreferencesFactory.create(walletId.id)
        val response = runCatching {
            transactionsService.getTransactions(walletId.id, null, preferences.transactionsTimestamp.toULong()).decodeJson<TransactionsResponse>()
        }.getOrNull() ?: return

        sync(wallet, response)
        preferences.transactionsTimestamp = currentTimestamp()
    }

    override suspend fun syncAssetTransactions(assetId: AssetId) {
        val wallet = sessionRepository.getCurrentWallet() ?: return

        syncAssetTransactions(wallet, assetId)
    }

    private suspend fun syncAssetTransactions(wallet: Wallet, assetId: AssetId) {
        val walletId = wallet.id
        val preferences = walletPreferencesFactory.create(walletId.id)
        val assetIdentifier = assetId.toIdentifier()
        val timestamp = preferences.transactionsForAssetTimestamp(assetIdentifier)
        val response = runCatching {
            transactionsService.getTransactions(walletId.id, assetIdentifier, timestamp.toULong()).decodeJson<TransactionsResponse>()
        }.getOrNull() ?: return

        sync(wallet, response)
        preferences.setTransactionsForAssetTimestamp(assetIdentifier, currentTimestamp())
    }

    private suspend fun sync(wallet: Wallet, response: TransactionsResponse) {
        val assetIds = response.transactions
            .flatMap { it.getAssociatedAssetIds() }
            .distinct()
        val newAssetIds = prefetchAssets.prefetchAssets(assetIds)
        assetsRepository.addBalancesIfMissing(wallet.id, newAssetIds)

        saveTransactions.saveTransactions(walletId = wallet.id, response.transactions)
        saveAddressNames.saveAddressNames(response.addressNames)
    }
}
