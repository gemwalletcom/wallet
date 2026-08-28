package com.gemwallet.android.data.coordinators.asset

import android.util.Log
import com.gemwallet.android.application.assets.coordinators.SyncAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.runCatchingCancellable
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope

class SyncAssetsImpl(
    private val sessionRepository: SessionRepository,
    private val deviceAssetsSyncService: DeviceAssetsSyncService,
    private val assetsRepository: AssetsRepository,
) : SyncAssets {
    override suspend fun invoke() = syncAssets()

    private suspend fun syncAssets() {
        coroutineScope {
            val walletId = sessionRepository.session().value?.wallet?.id?.id
            val balances = async {
                runCatchingCancellable { assetsRepository.sync() }
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
