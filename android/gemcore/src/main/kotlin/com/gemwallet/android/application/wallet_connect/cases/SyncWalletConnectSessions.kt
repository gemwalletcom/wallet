package com.gemwallet.android.application.wallet_connect.cases

import com.wallet.core.primitives.Wallet

interface SyncWalletConnectSessions {
    suspend fun syncSessions()

    fun followWallet(wallet: Wallet)
}
