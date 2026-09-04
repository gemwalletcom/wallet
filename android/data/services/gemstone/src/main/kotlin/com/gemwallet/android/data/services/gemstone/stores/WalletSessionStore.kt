package com.gemwallet.android.data.services.gemstone.stores

import kotlinx.coroutines.flow.Flow
import uniffi.gemstone.GemWalletSessionStore

class GemstoneWalletSessionStore(
    private val preferences: GemstonePreferencesStore,
) : GemWalletSessionStore {

    override fun getCurrentWalletId(): String? = preferences.get(CURRENT_WALLET_ID)

    override fun setCurrentWalletId(walletId: String?) {
        when (walletId) {
            null -> preferences.remove(CURRENT_WALLET_ID)
            else -> preferences.set(CURRENT_WALLET_ID, walletId)
        }
    }

    fun observeWalletId(): Flow<String?> = preferences.observe(CURRENT_WALLET_ID)

    private companion object {
        const val CURRENT_WALLET_ID = "current_wallet_id"
    }
}
