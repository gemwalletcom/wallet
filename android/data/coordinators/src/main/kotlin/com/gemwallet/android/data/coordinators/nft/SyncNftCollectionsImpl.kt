package com.gemwallet.android.data.coordinators.nft

import com.gemwallet.android.application.nft.coordinators.SyncNftCollections
import com.gemwallet.android.cases.nft.SyncNfts
import com.wallet.core.primitives.WalletId

class SyncNftCollectionsImpl(
    private val syncNfts: SyncNfts,
) : SyncNftCollections {

    override suspend fun syncNftCollections(walletId: WalletId) {
        runCatching { syncNfts.sync(walletId) }
    }
}
