package com.gemwallet.android.application.wallet_import.cases

import com.wallet.core.primitives.Wallet

interface SetupWallet {
    suspend fun setup(wallet: Wallet)
}
