package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.ConnectionsDao
import com.gemwallet.android.data.services.store.database.WalletsDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.wallet.core.primitives.WalletConnection
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
class ConnectionsQuery @Inject constructor(private val walletsDao: WalletsDao, private val connectionsDao: ConnectionsDao) {

    operator fun invoke(): Flow<List<WalletConnection>> = walletsDao.getAll().toDTO().flatMapLatest { wallets ->
        connectionsDao.getAll().map { records -> records.mapNotNull { it.toDTO(wallets) } }
    }
}
