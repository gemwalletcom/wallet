package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.entities.DbRecentActivity
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.RecentAsset
import com.gemwallet.android.model.RecentAssetsRequest
import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import javax.inject.Inject
import javax.inject.Singleton

@OptIn(ExperimentalCoroutinesApi::class)
@Singleton
class RecentAssetsService @Inject constructor(
    private val assetsDao: AssetsDao,
    private val sessionRepository: SessionRepository,
) {

    suspend fun addRecentActivity(
        assetId: AssetId,
        walletId: String,
        type: RecentType,
        toAssetId: AssetId? = null,
    ) {
        return assetsDao.addRecentActivity(
            DbRecentActivity(
                assetId = assetId.toIdentifier(),
                walletId = walletId,
                toAssetId = toAssetId?.toIdentifier(),
                type = type,
                addedAt = System.currentTimeMillis(),
            )
        )
    }

    fun getRecentAssets(request: RecentAssetsRequest): Flow<List<RecentAsset>> {
        return sessionRepository.currentWalletId()
            .flatMapLatest { walletId -> assetsDao.getRecentAssets(walletId, request.types, request.filters, request.limit) }
            .map { items ->
                items.mapNotNull { row ->
                    val asset = row.asset.toDTO() ?: return@mapNotNull null
                    RecentAsset(asset = asset, addedAt = row.addedAt)
                }
            }
    }

    suspend fun clearRecentAssets(walletId: WalletId, types: List<RecentType>) {
        assetsDao.clearRecentAssets(walletId.id, types)
    }
}
