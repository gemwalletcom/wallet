package com.gemwallet.android.data.coordinators.nft

import android.util.Log
import com.gemwallet.android.application.nft.coordinators.SyncNftCollections
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.ext.runCatchingCancellable
import com.wallet.core.primitives.WalletId

class SyncNftCollectionsImpl(
    private val syncNfts: SyncNfts,
) : SyncNftCollections {

    override suspend fun syncNftCollections(walletId: WalletId) {
        runCatchingCancellable { syncNfts.sync(walletId) }
            .onFailure { Log.e(TAG, "nft collections sync failed for ${walletId.id}", it) }
    }

    private companion object {
        const val TAG = "SyncNftCollections"
    }
}
