package com.gemwallet.android.application.wallet_import.cases

import com.gemwallet.android.application.wallet_import.values.WalletImportResult
import com.gemwallet.android.model.ImportType
import com.wallet.core.primitives.Wallet

interface ImportWalletService {
    suspend fun importWallet(
        importType: ImportType,
        walletName: String,
        data: String,
    ): WalletImportResult

    suspend fun createWallet(
        walletName: String,
        data: String,
    ): Wallet
}
