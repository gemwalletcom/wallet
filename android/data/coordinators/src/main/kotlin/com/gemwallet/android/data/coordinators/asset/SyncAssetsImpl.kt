package com.gemwallet.android.data.coordinators.asset

import android.util.Log
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.assets.cases.SyncAssets
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemWalletHomeServiceInterface

class SyncAssetsImpl(
    private val getSession: GetSession,
    private val getWalletAssets: GetWalletAssets,
    private val homeService: GemWalletHomeServiceInterface,
) : SyncAssets {

    override suspend fun invoke() = withContext(Dispatchers.IO) {
        val wallet = getSession().value?.wallet ?: return@withContext
        val assetIds = getWalletAssets(wallet.id).firstOrNull().orEmpty().map { it.asset.id.toIdentifier() }
        runCatchingCancellable { homeService.refresh(wallet.id.id, assetIds) }
            .onFailure { Log.e(TAG, "assets refresh failed for ${wallet.id.id}", it) }
        Unit
    }

    private companion object {
        const val TAG = "SyncAssets"
    }
}
