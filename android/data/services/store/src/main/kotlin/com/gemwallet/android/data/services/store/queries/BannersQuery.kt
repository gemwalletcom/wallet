package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.BannersDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.BannerEvent
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class BannersQuery @Inject constructor(private val bannersDao: BannersDao) {

    operator fun invoke(walletId: String?, assetId: AssetId): Flow<List<Banner>> = bannersDao.observeAssetBanners(walletId, assetId.toIdentifier(), AssetId(assetId.chain).toIdentifier())
        .map { records -> records.map { it.toDTO() } }

    operator fun invoke(walletId: String, events: List<BannerEvent>): Flow<List<Banner>> = bannersDao.observeWalletBanners(walletId, events)
        .map { records -> records.map { it.toDTO() } }
}
