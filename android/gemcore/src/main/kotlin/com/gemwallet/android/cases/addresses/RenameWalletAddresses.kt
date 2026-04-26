package com.gemwallet.android.cases.addresses

interface RenameWalletAddresses {
    suspend fun renameWalletAddresses(walletId: String, name: String)
}
