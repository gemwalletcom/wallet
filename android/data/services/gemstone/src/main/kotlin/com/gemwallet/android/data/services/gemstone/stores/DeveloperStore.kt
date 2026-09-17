package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BannersDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.StakeDao
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.AssetType
import uniffi.gemstone.GemDeveloperStore

class GemstoneDeveloperStore(
    private val transactionsDao: TransactionsDao,
    private val assetsDao: AssetsDao,
    private val stakeDao: StakeDao,
    private val bannersDao: BannersDao,
    private val pricesDao: PricesDao,
) : GemDeveloperStore {

    override suspend fun clearTransactions() = transactionsDao.deleteAll()

    override suspend fun clearTokens() = assetsDao.deleteTokens(AssetType.NATIVE)

    override suspend fun clearDelegations() = stakeDao.deleteAllDelegations()

    override suspend fun clearValidators() = stakeDao.deleteAllValidators()

    override suspend fun clearPrices() = pricesDao.deleteAll()

    override suspend fun clearBanners() = bannersDao.deleteAll()

    override suspend fun updateBannerStates(from: uniffi.gemstone.BannerState, to: uniffi.gemstone.BannerState) =
        bannersDao.updateStates(from.toPrimitives(), to.toPrimitives())
}
