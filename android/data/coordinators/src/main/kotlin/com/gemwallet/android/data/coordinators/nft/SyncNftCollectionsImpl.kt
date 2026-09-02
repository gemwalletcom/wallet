package com.gemwallet.android.data.coordinators.nft

import android.util.Log
import com.gemwallet.android.application.nft.cases.SyncNftCollections
import com.gemwallet.android.ext.runCatchingCancellable
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemNftService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class SyncNftCollectionsImpl(
    private val nftService: GemNftService,
) : SyncNftCollections {

    override suspend fun syncNftCollections(walletId: WalletId) = withContext(Dispatchers.IO) {
        runCatchingCancellable { nftService.sync(walletId.id) }
            .onFailure { Log.e(TAG, "nft collections sync failed for ${walletId.id}", it) }
        Unit
    }

    private companion object {
        const val TAG = "SyncNftCollections"
    }
}
