package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.AssetListDao
import com.gemwallet.android.data.services.store.database.SearchDao
import com.gemwallet.android.data.services.store.database.entities.toAssetSearchRecords
import com.gemwallet.android.data.services.store.database.entities.toListSearchRecords
import com.gemwallet.android.data.services.store.database.entities.toPerpetualSearchRecords
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.GemSearchStore

class GemstoneSearchStore(private val searchDao: SearchDao, private val assetListDao: AssetListDao) : GemSearchStore {
    override suspend fun setAssets(key: String, assetIds: List<String>) = searchDao.putAssets(key, assetIds.toAssetSearchRecords(key))

    override suspend fun setPerpetuals(key: String, perpetualIds: List<String>) = searchDao.putPerpetuals(key, perpetualIds.toPerpetualSearchRecords(key))

    override suspend fun setLists(key: String, lists: List<uniffi.gemstone.AssetList>) {
        val items = lists.map { it.toPrimitives() }
        assetListDao.upsert(items.toRecord())
        searchDao.putLists(key, items.map { it.id }.toListSearchRecords(key))
    }
}
