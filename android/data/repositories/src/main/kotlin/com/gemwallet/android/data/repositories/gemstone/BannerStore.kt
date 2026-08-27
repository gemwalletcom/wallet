package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.BannersDao
import com.gemwallet.android.data.service.store.database.entities.DbBanner
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.Chain
import uniffi.gemstone.BannerState
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemBannerStore
import uniffi.gemstone.bannerIdentifier

class GemstoneBannerStore(
    private val bannersDao: BannersDao,
) : GemBannerStore {

    override suspend fun getState(key: GemBannerKey): BannerState? =
        bannersDao.getBanner(bannerIdentifier(key))?.state?.toJson()

    override suspend fun setState(key: GemBannerKey, state: BannerState) {
        val record = key.toRecord(state)
        if (bannersDao.getBanner(record.id)?.state != record.state) {
            bannersDao.saveBanner(record)
        }
    }

    override suspend fun addBanners(keys: List<GemBannerKey>, state: BannerState) {
        bannersDao.addBanners(keys.map { it.toRecord(state) })
    }

    private fun GemBannerKey.toRecord(state: BannerState) = DbBanner(
        id = bannerIdentifier(this),
        walletId = walletId,
        assetId = assetId,
        chain = chain?.let { chain -> Chain.entries.firstOrNull { it.string == chain } },
        event = event.decodeJson<BannerEvent>(),
        state = state.decodeJson(),
    )
}
