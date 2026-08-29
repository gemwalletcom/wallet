package com.gemwallet.android.application.wallet_import.values

import com.wallet.core.primitives.Wallet

sealed class WalletImportResult {
    abstract val wallet: Wallet

    data class New(override val wallet: Wallet) : WalletImportResult()
    data class Existing(override val wallet: Wallet) : WalletImportResult()
}
