package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.BannersDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.wallet.core.primitives.Banner
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class BannersQuery @Inject constructor(private val bannersDao: BannersDao) {

    operator fun invoke(): Flow<List<Banner>> = bannersDao.observeBanners()
        .map { records -> records.map { it.toDTO() } }
}
