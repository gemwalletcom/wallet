package com.gemwallet.android.data.coordinators.banner

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.banner.cases.GetActiveBanners
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.services.gemstone.stores.GemstoneBannerStore
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.toGem
import com.wallet.core.primitives.Asset
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import uniffi.gemstone.GemBannerRow
import uniffi.gemstone.assetBannerContext

@OptIn(ExperimentalCoroutinesApi::class)
class GetActiveBannersImpl(private val getSession: GetSession, private val getAssetInfo: GetAssetInfo, private val bannerStore: GemstoneBannerStore) : GetActiveBanners {

    override fun invoke(asset: Asset): Flow<List<GemBannerRow>> = getSession()
        .flatMapLatest { session ->
            val wallet = session?.wallet
            combine(
                bannerStore.observeAssetBanners(wallet?.id?.id, asset.id),
                getAssetInfo(asset.id),
            ) { records, assetInfo ->
                assetInfo?.let {
                    assetBannerContext(wallet?.toGem(), it.asset.toGem(), it.metadata.toGem(), it.balance.toGem())
                        .visibleBanners(stored = records.map { record -> record.toDTO().toGem() })
                }.orEmpty()
            }
        }
        .flowOn(Dispatchers.IO)
}
