package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.WalletsDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject

class WalletsQuery @Inject constructor(private val walletsDao: WalletsDao) {

    operator fun invoke(): Flow<List<Wallet>> = walletsDao.getAll().toDTO()
}
