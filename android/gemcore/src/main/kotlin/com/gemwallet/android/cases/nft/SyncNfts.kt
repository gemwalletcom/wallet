package com.gemwallet.android.cases.nft

interface SyncNfts {
    suspend fun sync(walletId: String)
}
