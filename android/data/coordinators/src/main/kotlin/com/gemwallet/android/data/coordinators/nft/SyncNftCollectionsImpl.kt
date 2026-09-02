package com.gemwallet.android.data.coordinators.nft

import android.util.Log
import com.gemwallet.android.application.nft.cases.SyncNftCollections
import com.gemwallet.android.ext.runCatchingCancellable
import uniffi.gemstone.GemNftService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class SyncNftCollectionsImpl(
    private val nftService: GemNftService,
) : SyncNftCollections {

    override suspend fun syncNftCollections() = withContext(Dispatchers.IO) {
        runCatchingCancellable { nftService.sync() }
            .onFailure { Log.e(TAG, "nft collections sync failed", it) }
        Unit
    }

    private companion object {
        const val TAG = "SyncNftCollections"
    }
}
