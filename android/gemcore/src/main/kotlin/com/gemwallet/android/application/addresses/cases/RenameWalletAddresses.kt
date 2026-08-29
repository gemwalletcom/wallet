package com.gemwallet.android.application.addresses.cases

import com.wallet.core.primitives.WalletId

interface RenameWalletAddresses {
    suspend fun rename(walletId: WalletId, name: String)
}
