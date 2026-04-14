package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.SyncAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import kotlinx.coroutines.CancellationException
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
            val walletId = sessionRepository.session().value?.wallet?.id
            val balances = async {
                assetsRepository.sync()
            }
            val deviceAssets = walletId?.let {
                async {
                    try {
                        deviceAssetsSyncService.sync(it)
                    } catch (cancelled: CancellationException) {
                        throw cancelled
                    } catch (_: Throwable) {
                    }
                }
            }

            balances.await()
            deviceAssets?.await()
        }
    }
}
