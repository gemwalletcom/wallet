package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import uniffi.gemstone.GemAssetDiscoveryStore

class GemstoneAssetDiscoveryStore(
    private val walletPreferencesFactory: WalletPreferencesFactory,
) : GemAssetDiscoveryStore {

    override suspend fun getAssetsTimestamp(walletId: String): ULong =
        walletPreferencesFactory.create(walletId).assetsTimestamp.toULong()

    override suspend fun setAssetsTimestamp(walletId: String, timestamp: ULong) {
        walletPreferencesFactory.create(walletId).assetsTimestamp = timestamp.toLong()
    }
}
