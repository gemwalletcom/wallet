package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import uniffi.gemstone.GemAssetDiscoveryStore
import uniffi.gemstone.GemDiscoveryStep

class GemstoneAssetDiscoveryStore(
    private val walletPreferencesFactory: WalletPreferencesFactory,
) : GemAssetDiscoveryStore {

    override suspend fun getAssetsTimestamp(walletId: String): ULong =
        walletPreferencesFactory.create(walletId).assetsTimestamp.toULong()

    override suspend fun setAssetsTimestamp(walletId: String, timestamp: ULong) {
        walletPreferencesFactory.create(walletId).assetsTimestamp = timestamp.toLong()
    }

    override suspend fun isCompleted(walletId: String, step: GemDiscoveryStep): Boolean =
        walletPreferencesFactory.create(walletId).isInitialLoadCompleted(step.name.lowercase())

    override suspend fun setCompleted(walletId: String, step: GemDiscoveryStep) =
        walletPreferencesFactory.create(walletId).setInitialLoadCompleted(step.name.lowercase())
}
