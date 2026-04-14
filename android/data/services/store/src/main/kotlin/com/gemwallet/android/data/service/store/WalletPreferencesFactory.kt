package com.gemwallet.android.data.service.store

import android.content.Context
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class WalletPreferencesFactory @Inject constructor(
    @ApplicationContext private val context: Context,
) {
    fun create(walletId: String): WalletPreferences = WalletPreferences(context, walletId)
}
