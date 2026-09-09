package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.BannersDao
import com.gemwallet.android.data.service.store.database.entities.DbBanner
import com.gemwallet.android.data.service.store.database.entities.DbBannerWithAsset
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.BannerEvent
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemBannerStore

class GemstoneBannerStore(
    private val bannersDao: BannersDao,
) : GemBannerStore {

    override suspend fun getState(key: GemBannerKey): uniffi.gemstone.BannerState? =
        bannersDao.getBanner(key.identifier())?.state?.toGem()

    override suspend fun setState(key: GemBannerKey, state: uniffi.gemstone.BannerState) {
        val record = key.toRecord(state)
        if (bannersDao.getBanner(record.id)?.state != record.state) {
            bannersDao.saveBanner(record)
        }
    }

    override suspend fun addBanners(keys: List<GemBannerKey>, state: uniffi.gemstone.BannerState) {
        bannersDao.addBanners(keys.map { it.toRecord(state) })
    }

    fun observeAssetBanners(walletId: String?, assetId: AssetId): Flow<List<DbBannerWithAsset>> =
        bannersDao.observeAssetBanners(walletId, assetId.toIdentifier(), AssetId(assetId.chain).toIdentifier())

    fun observeWalletBanners(walletId: String, events: List<BannerEvent>): Flow<List<DbBannerWithAsset>> = bannersDao.observeWalletBanners(walletId, events)

    private fun GemBannerKey.toRecord(state: uniffi.gemstone.BannerState) = DbBanner(
        id = identifier(),
        walletId = walletId,
        assetId = assetId,
        event = event.toPrimitives(),
        state = state.toPrimitives(),
    )
}
