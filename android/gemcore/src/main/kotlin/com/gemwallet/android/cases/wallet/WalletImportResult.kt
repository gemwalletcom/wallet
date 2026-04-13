package com.gemwallet.android.cases.wallet

import com.wallet.core.primitives.Wallet

sealed class WalletImportResult {
    abstract val wallet: Wallet

    data class New(override val wallet: Wallet) : WalletImportResult()
    data class Existing(override val wallet: Wallet) : WalletImportResult()
}
