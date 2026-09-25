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
class ConnectionQuery @Inject constructor(private val walletsDao: WalletsDao, private val connectionsDao: ConnectionsDao) {

    operator fun invoke(connectionId: String): Flow<WalletConnection?> = walletsDao.getAll().toDTO().flatMapLatest { wallets ->
        connectionsDao.getConnection(connectionId).map { it?.toDTO(wallets) }
    }
}
