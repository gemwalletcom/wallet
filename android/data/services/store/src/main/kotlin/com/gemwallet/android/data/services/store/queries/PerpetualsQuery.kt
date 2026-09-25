package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.PerpetualDao
import com.gemwallet.android.data.services.store.database.SearchDao
import com.gemwallet.android.data.services.store.database.entities.DbPerpetualData
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.wallet.core.primitives.PerpetualData
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
class PerpetualsQuery @Inject constructor(private val perpetualDao: PerpetualDao, private val searchDao: SearchDao) {

    operator fun invoke(searchQuery: String, limit: Int, requiresVolume: Boolean): Flow<List<PerpetualData>> {
        if (searchQuery.isEmpty()) {
            return perpetualDao.getPerpetualsData(requiresVolume, limit).toPerpetualData()
        }
        return searchDao.hasPerpetualPriorities(searchQuery)
            .map { it > 0 }
            .distinctUntilChanged()
            .flatMapLatest { hasPriority ->
                when (hasPriority) {
                    true -> perpetualDao.searchWithPriority(searchQuery, limit).toPerpetualData()
                    false -> perpetualDao.searchPerpetualsData(searchQuery, limit).toPerpetualData()
                }
            }
    }

    private fun Flow<List<DbPerpetualData>>.toPerpetualData(): Flow<List<PerpetualData>> = map { items -> items.mapNotNull { it.toDTO() } }
}
