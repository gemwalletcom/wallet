package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.PerpetualDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.wallet.core.primitives.PerpetualData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class PerpetualsQuery @Inject constructor(private val perpetualDao: PerpetualDao) {

    operator fun invoke(searchQuery: String, limit: Int, requiresVolume: Boolean): Flow<List<PerpetualData>> = when (searchQuery.isEmpty()) {
        true -> perpetualDao.getPerpetualsData(requiresVolume, limit)
        false -> perpetualDao.searchPerpetualsData(searchQuery, limit)
    }.map { items -> items.mapNotNull { it.toDTO() } }
}
