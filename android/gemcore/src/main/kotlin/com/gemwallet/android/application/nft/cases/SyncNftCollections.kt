package com.gemwallet.android.application.nft.cases

import com.wallet.core.primitives.WalletId

interface SyncNftCollections {
    suspend fun syncNftCollections(walletId: WalletId)
}
