package com.gemwallet.android.cases.nft

import com.wallet.core.primitives.Wallet

interface SyncNfts {
    suspend fun syncNfts(wallet: Wallet)
}
