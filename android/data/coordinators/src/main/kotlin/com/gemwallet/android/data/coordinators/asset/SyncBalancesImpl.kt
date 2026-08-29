package com.gemwallet.android.data.coordinators.asset

import android.util.Log
import com.gemwallet.android.application.assets.cases.SyncBalances
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemBalanceService

class SyncBalancesImpl(
    private val balanceService: GemBalanceService,
) : SyncBalances {

    override suspend fun invoke(assets: List<AssetInfo>) = withContext(Dispatchers.IO) {
        assets.groupBy { it.walletId }
            .mapNotNull { (walletId, assetInfos) ->
                walletId ?: return@mapNotNull null
                async {
                    runCatchingCancellable { balanceService.update(walletId.id, assetInfos.map { it.asset.id.toIdentifier() }) }
                        .onFailure { Log.e(TAG, "balances update failed for ${walletId.id}", it) }
                    Unit
                }
            }
            .awaitAll()
        Unit
    }

    private companion object {
        const val TAG = "SyncBalances"
    }
}
