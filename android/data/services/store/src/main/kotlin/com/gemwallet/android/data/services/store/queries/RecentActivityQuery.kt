package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.application.assets.values.AssetsQueryFilter
import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.model.RecentAsset
import com.wallet.core.primitives.RecentActivityType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class RecentActivityQuery @Inject constructor(private val assetsDao: AssetsDao) {

    operator fun invoke(walletId: WalletId, types: List<RecentActivityType> = RecentActivityType.entries, filters: Set<AssetsQueryFilter> = emptySet(), limit: Int = GemConstants.recentAssetsLimit): Flow<List<RecentAsset>> =
        assetsDao.getRecentAssets(walletId.id, types, filters, limit)
            .map { rows -> rows.mapNotNull { row -> row.asset.toDTO()?.let { RecentAsset(asset = it, addedAt = row.addedAt) } } }
}
