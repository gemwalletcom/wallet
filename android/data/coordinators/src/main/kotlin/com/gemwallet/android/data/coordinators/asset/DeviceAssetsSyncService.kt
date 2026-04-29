package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.gemwallet.android.application.assets.coordinators.EnsureWalletAssets
import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.firstOrNull
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class DeviceAssetsSyncService @Inject constructor(
    private val walletPreferencesFactory: WalletPreferencesFactory,
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val prefetchAssets: PrefetchAssets,
    private val ensureWalletAssets: EnsureWalletAssets,
    private val enableAsset: EnableAsset,
    private val assetsRepository: AssetsRepository,
    private val walletsRepository: WalletsRepository,
) {

    suspend fun sync(walletId: String) {
        val preferences = walletPreferencesFactory.create(walletId)
        val assetIds = gemDeviceApiClient.getAssets(
            walletId = walletId,
            fromTimestamp = preferences.assetsTimestamp,
        ).mapNotNull(String::toAssetId)
            .distinct()

        if (assetIds.isEmpty()) {
            preferences.assetsTimestamp = currentTimestamp()
            return
        }

        val wallet = walletsRepository.getWallet(walletId).firstOrNull() ?: return
        val existingAssetIds = assetsRepository.hasWalletAssets(wallet.id, assetIds)
        val missingAssetIds = assetIds.filterNot(existingAssetIds::contains)

        prefetchAssets.prefetchAssets(assetIds)
        enableAssets(wallet, existingAssetIds)

        if (missingAssetIds.isNotEmpty()) {
            ensureWalletAssets.ensureWalletAssets(wallet, missingAssetIds)
        }

        if (missingAssetIds.isEmpty() ||
            assetsRepository.hasWalletAssets(wallet.id, missingAssetIds).containsAll(missingAssetIds)
        ) {
            preferences.assetsTimestamp = currentTimestamp()
        }
    }

    private suspend fun enableAssets(wallet: Wallet, assetIds: Collection<AssetId>) {
        val accounts = assetIds.filter { wallet.getAccount(it) != null }
        if (accounts.isEmpty()) return
        enableAsset(wallet.id, accounts)
    }

    private fun currentTimestamp(): Long = System.currentTimeMillis() / 1000
}
