package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.AssetListDao
import com.gemwallet.android.data.service.store.database.SearchDao
import com.gemwallet.android.data.service.store.database.entities.DbSearch
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.AssetList
import uniffi.gemstone.GemSearchStore

class GemstoneSearchStore(
    private val searchDao: SearchDao,
    private val assetListDao: AssetListDao,
) : GemSearchStore {
    override suspend fun setAssets(key: String, assetIds: List<String>) =
        searchDao.putAssets(key, assetIds.mapIndexed { index, id -> DbSearch(query = key, assetId = id, priority = index) })

    override suspend fun setPerpetuals(key: String, perpetualIds: List<String>) =
        searchDao.putPerpetuals(key, perpetualIds.mapIndexed { index, id -> DbSearch(query = key, perpetualId = id, priority = index) })

    override suspend fun setLists(key: String, lists: List<String>) {
        val items = lists.map { it.decodeJson<AssetList>() }
        assetListDao.upsert(items.toRecord())
        searchDao.putLists(key, items.mapIndexed { index, list -> DbSearch(query = key, listId = list.id, priority = index) })
    }
}
