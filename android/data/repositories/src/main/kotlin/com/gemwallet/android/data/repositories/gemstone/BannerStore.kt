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

class GemstoneBannerStore(
    private val bannersDao: BannersDao,
) : GemBannerStore {

    override suspend fun getState(key: GemBannerKey): BannerState? =
        bannersDao.getBanner(
            walletId = key.walletId.orEmpty(),
            assetId = key.assetId.orEmpty(),
            chain = key.chain,
            event = key.event.decodeJson<BannerEvent>(),
        )?.state?.toJson()

    override suspend fun setState(key: GemBannerKey, state: BannerState) {
        bannersDao.saveBanner(key.toRecord(state))
    }

    override suspend fun addBanners(keys: List<GemBannerKey>, state: BannerState) {
        keys.filter { getState(it) == null }.forEach { bannersDao.saveBanner(it.toRecord(state)) }
    }

    private fun GemBannerKey.toRecord(state: BannerState) = DbBanner(
        walletId = walletId.orEmpty(),
        assetId = assetId.orEmpty(),
        chain = chain?.let { chain -> Chain.entries.firstOrNull { it.string == chain } },
        event = event.decodeJson<BannerEvent>(),
        state = state.decodeJson(),
    )
}
