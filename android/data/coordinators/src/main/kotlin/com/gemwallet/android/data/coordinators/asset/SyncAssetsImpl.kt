package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.SyncAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
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
            val balances = async { runCatching { assetsRepository.sync() } }
            val deviceAssets = walletId?.let {
                async { runCatching { deviceAssetsSyncService.sync(it) } }
            }

            balances.await()
            deviceAssets?.await()
        }
    }
}
