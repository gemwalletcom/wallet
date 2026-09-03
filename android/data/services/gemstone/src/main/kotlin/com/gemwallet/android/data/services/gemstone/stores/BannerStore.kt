package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.BannersDao
import com.gemwallet.android.data.service.store.database.entities.DbBanner
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.BannerEvent
import uniffi.gemstone.BannerState
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemBannerStore
import com.gemwallet.android.ext.requireChain
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class GemstoneBannerStore(
    private val bannersDao: BannersDao,
) : GemBannerStore {

    override suspend fun getState(key: GemBannerKey): BannerState? =
        bannersDao.getBanner(key.identifier())?.state?.toJson()

    override suspend fun setState(key: GemBannerKey, state: BannerState) {
        val record = key.toRecord(state)
        if (bannersDao.getBanner(record.id)?.state != record.state) {
            bannersDao.saveBanner(record)
        }
    }

    override suspend fun addBanners(keys: List<GemBannerKey>, state: BannerState) {
        bannersDao.addBanners(keys.map { it.toRecord(state) })
    }

    fun observeAssetBanners(walletId: String?, assetId: String): Flow<List<DbBanner>> = bannersDao.observeAssetBanners(walletId, assetId)

    fun observeWalletBanners(walletId: String, events: List<BannerEvent>): Flow<List<DbBanner>> = bannersDao.observeWalletBanners(walletId, events)

    fun observeMultiSign(walletId: String): Flow<Boolean> = bannersDao.getMultisign(walletId).map { it.isNotEmpty() }

    private fun GemBannerKey.toRecord(state: BannerState) = DbBanner(
        id = identifier(),
        walletId = walletId,
        assetId = assetId,
        event = event.decodeJson<BannerEvent>(),
        state = state.decodeJson(),
    )
}
