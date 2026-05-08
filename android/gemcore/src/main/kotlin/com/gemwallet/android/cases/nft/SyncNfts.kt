package com.gemwallet.android.cases.nft

import com.wallet.core.primitives.WalletId

interface SyncNfts {
    suspend fun sync(walletId: WalletId)
}
