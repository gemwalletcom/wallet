package com.gemwallet.android.data.coordinators.nft

import android.util.Log
import com.gemwallet.android.application.nft.cases.SyncNftCollections
import com.gemwallet.android.ext.runCatchingCancellable
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemNftService

class SyncNftCollectionsImpl(
    private val nftService: GemNftService,
) : SyncNftCollections {

    override suspend fun syncNftCollections(walletId: WalletId) {
        runCatchingCancellable { nftService.sync(walletId.id) }
            .onFailure { Log.e(TAG, "nft collections sync failed for ${walletId.id}", it) }
    }

    private companion object {
        const val TAG = "SyncNftCollections"
    }
}
