package com.gemwallet.android.data.coordinators.banner

import com.gemwallet.android.application.banner.cases.GetAssetBanners
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.data.services.gemstone.stores.GemstoneBannerStore
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map

@OptIn(ExperimentalCoroutinesApi::class)
class GetAssetBannersImpl(private val getSession: GetSession, private val bannerStore: GemstoneBannerStore) : GetAssetBanners {

    override fun invoke(asset: Asset): Flow<List<Banner>> = getSession()
        .flatMapLatest { session -> bannerStore.observeAssetBanners(session?.wallet?.id?.id, asset.id) }
        .map { records -> records.map { it.toDTO() } }
        .flowOn(Dispatchers.IO)
}
