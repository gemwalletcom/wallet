package com.gemwallet.android.data.repositories.gemstone

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
    override suspend fun setAssets(key: String, assetIds: List<String>) {
        if (assetIds.isEmpty()) {
            searchDao.deleteAssets(key)
        } else {
            searchDao.put(assetIds.mapIndexed { index, id -> DbSearch(query = key, assetId = id, priority = index) })
        }
    }

    override suspend fun setPerpetuals(key: String, perpetualIds: List<String>) {
        if (perpetualIds.isEmpty()) {
            searchDao.deletePerpetuals(key)
        } else {
            searchDao.put(perpetualIds.mapIndexed { index, id -> DbSearch(query = key, perpetualId = id, priority = index) })
        }
    }

    override suspend fun setLists(key: String, lists: List<String>) {
        val items = lists.map { it.decodeJson<AssetList>() }
        if (items.isEmpty()) {
            searchDao.deleteLists(key)
        } else {
            assetListDao.upsert(items.toRecord())
            searchDao.put(items.mapIndexed { index, list -> DbSearch(query = key, listId = list.id, priority = index) })
        }
    }
}
