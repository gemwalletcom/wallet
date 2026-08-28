package com.gemwallet.android.data.coordinators.asset

import android.util.Log
import com.gemwallet.android.application.assets.coordinators.SyncMissingAssets
import com.gemwallet.android.application.assets.coordinators.SyncAssetInfo
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetFull
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemBalanceService
import uniffi.gemstone.GemStreamSubscriptionService

class SyncAssetInfoImpl(
    private val assetsService: GemAssetsService,
    private val balanceService: GemBalanceService,
    private val streamSubscriptionService: GemStreamSubscriptionService,
    private val syncMissingAssets: SyncMissingAssets,
    private val sessionRepository: SessionRepository,
) : SyncAssetInfo {

    override suspend fun syncAssetInfo(assetId: AssetId, wallet: Wallet): Unit = withContext(Dispatchers.IO) {
        wallet.getAccount(assetId) ?: return@withContext

        streamSubscriptionService.addPrices(listOf(assetId.toIdentifier()))

        val assetFull = syncAssetMetadata(assetId)
        coroutineScope {
            launch { syncBalance(wallet, assetId) }
            assetFull?.let { launch { syncMissingAssets.syncMissingAssets(it.associations.map { association -> association.assetId }) } }
        }
    }

    private suspend fun syncBalance(wallet: Wallet, assetId: AssetId) {
        try {
            balanceService.update(wallet.id.id, listOf(assetId.toIdentifier()))
        } catch (error: Exception) {
            currentCoroutineContext().ensureActive()
            Log.e(TAG, "balance update failed for ${assetId.toIdentifier()}", error)
        }
    }

    private suspend fun syncAssetMetadata(assetId: AssetId): AssetFull? {
        return try {
            assetsService.syncAsset(assetId.toIdentifier(), sessionRepository.getCurrentCurrency().toJson()).decodeJson<AssetFull>()
        } catch (_: Exception) {
            currentCoroutineContext().ensureActive()
            null
        }
    }

    private companion object {
        const val TAG = "SyncAssetInfo"
    }
}
