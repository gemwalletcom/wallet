package com.gemwallet.android.data.coordinators.asset

import android.util.Log
import com.gemwallet.android.application.assets.cases.SyncAssets
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.assets.cases.SyncBalances
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.runCatchingCancellable
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.firstOrNull

class SyncAssetsImpl(
    private val sessionRepository: SessionRepository,
    private val deviceAssetsSyncService: DeviceAssetsSyncService,
    private val getWalletAssets: GetWalletAssets,
    private val syncBalances: SyncBalances,
) : SyncAssets {
    override suspend fun invoke() = syncAssets()

    private suspend fun syncAssets() {
        coroutineScope {
            val walletId = sessionRepository.session().value?.wallet?.id?.id
            val balances = async {
                runCatchingCancellable { syncBalances(getWalletAssets().firstOrNull().orEmpty()) }
                    .onFailure { Log.e(TAG, "assets sync failed", it) }
            }
            val deviceAssets = walletId?.let { id ->
                async {
                    runCatchingCancellable { deviceAssetsSyncService.sync(id) }
                        .onFailure { Log.e(TAG, "device assets sync failed for $id", it) }
                }
            }

            balances.await()
            deviceAssets?.await()
        }
    }

    private companion object {
        const val TAG = "SyncAssets"
    }
}
