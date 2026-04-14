package com.gemwallet.android.data.service.store

import android.content.Context

class WalletPreferences(context: Context, walletId: String) {

    private val store = ConfigStore(
        store = context.getSharedPreferences(
            "wallet_preferences_$walletId",
            Context.MODE_PRIVATE,
        )
    )

    var transactionsTimestamp: Long
        get() = store.getLong(KEY_TRANSACTIONS_TIMESTAMP)
        set(value) = store.putLong(KEY_TRANSACTIONS_TIMESTAMP, value)

    var assetsTimestamp: Long
        get() = store.getLong(KEY_ASSETS_TIMESTAMP)
        set(value) = store.putLong(KEY_ASSETS_TIMESTAMP, value)

    fun transactionsForAssetTimestamp(assetId: String): Long {
        return store.getLong(KEY_TRANSACTIONS_FOR_ASSET, postfix = assetId)
    }

    fun setTransactionsForAssetTimestamp(assetId: String, value: Long) {
        store.putLong(KEY_TRANSACTIONS_FOR_ASSET, value, postfix = assetId)
    }

    fun clear() {
        store.store.edit().clear().apply()
    }

    companion object {
        private const val KEY_ASSETS_TIMESTAMP = "assets_timestamp"
        private const val KEY_TRANSACTIONS_TIMESTAMP = "transactions_timestamp"
        private const val KEY_TRANSACTIONS_FOR_ASSET = "transactions_for_asset"
    }
}
