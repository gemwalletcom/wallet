package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.ext.toAssetId
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemAssetDiscoveryStore

class GemstoneAssetDiscoveryStore(
    private val walletPreferencesFactory: WalletPreferencesFactory,
    private val enableAsset: EnableAsset,
) : GemAssetDiscoveryStore {

    override suspend fun getAssetsTimestamp(walletId: String): ULong =
        walletPreferencesFactory.create(walletId).assetsTimestamp.toULong()

    override suspend fun setAssetsTimestamp(walletId: String, timestamp: ULong) {
        walletPreferencesFactory.create(walletId).assetsTimestamp = timestamp.toLong()
    }

    override suspend fun enableAssets(walletId: String, assetIds: List<String>) =
        enableAsset(WalletId(walletId), assetIds.mapNotNull { it.toAssetId() })
}
