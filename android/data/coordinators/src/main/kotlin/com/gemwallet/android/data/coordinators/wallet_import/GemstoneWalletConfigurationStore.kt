package com.gemwallet.android.data.coordinators.wallet_import

import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import uniffi.gemstone.GemWalletConfigurationStore

class GemstoneWalletConfigurationStore(
    private val walletPreferencesFactory: WalletPreferencesFactory,
) : GemWalletConfigurationStore {

    override suspend fun isCompleted(walletId: String): Boolean =
        walletPreferencesFactory.create(walletId).completeInitialWalletConfiguration

    override suspend fun setCompleted(walletId: String) {
        walletPreferencesFactory.create(walletId).completeInitialWalletConfiguration = true
    }
}
