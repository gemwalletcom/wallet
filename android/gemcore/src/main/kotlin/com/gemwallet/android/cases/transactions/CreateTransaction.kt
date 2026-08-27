package com.gemwallet.android.cases.transactions

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId

interface CreateTransaction {
    suspend fun createTransaction(walletId: WalletId, transaction: Transaction, currency: Currency): Transaction

    suspend fun createNotificationTransaction(wallet: Wallet, assetId: AssetId, transaction: Transaction, currency: Currency): Asset?
}
