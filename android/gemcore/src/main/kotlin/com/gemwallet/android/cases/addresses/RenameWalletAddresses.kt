package com.gemwallet.android.cases.addresses

import com.wallet.core.primitives.WalletId

interface RenameWalletAddresses {
    suspend fun rename(walletId: WalletId, name: String)
}
