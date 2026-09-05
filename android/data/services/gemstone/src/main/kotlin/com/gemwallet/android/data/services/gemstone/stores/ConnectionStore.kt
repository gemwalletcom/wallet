package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.data.service.store.database.ConnectionsDao
import com.gemwallet.android.data.service.store.database.entities.DbConnection
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toSession
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletConnection
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemConnectionStore

@OptIn(ExperimentalCoroutinesApi::class)
class GemstoneConnectionStore(
    private val walletStore: GemstoneWalletStore,
    private val connectionsDao: ConnectionsDao,
) : GemConnectionStore {

    fun observeConnections(): Flow<List<WalletConnection>> = walletStore.observeWallets().flatMapLatest { wallets ->
        connectionsDao.getAll().map { records -> records.mapNotNull { it.toConnection(wallets) } }
    }

    fun observeConnection(connectionId: String): Flow<WalletConnection?> = walletStore.observeWallets().flatMapLatest { wallets ->
        connectionsDao.getConnection(connectionId).map { it?.toConnection(wallets) }
    }

    suspend fun getConnectionBySessionId(sessionId: String): WalletConnection? {
        val record = connectionsDao.getBySessionId(sessionId) ?: return null
        return record.toConnection(walletStore.observeWallets().firstOrNull().orEmpty())
    }

    override suspend fun getConnection(sessionId: String): uniffi.gemstone.WalletConnection? = getConnectionBySessionId(sessionId)?.toGem()

    override suspend fun getSessions(): List<uniffi.gemstone.WalletConnectionSession> = connectionsDao.getConnections().map { it.toSession().toGem() }

    override suspend fun addConnection(connection: uniffi.gemstone.WalletConnection) = connectionsDao.insert(connection.toPrimitives().toRecord())

    override suspend fun updateSession(session: uniffi.gemstone.WalletConnectionSession) {
        val updated = session.toPrimitives()
        val record = connectionsDao.getBySessionId(updated.id) ?: return
        connectionsDao.insert(
            record.copy(
                state = updated.state,
                chains = updated.chains,
                expireAt = updated.expireAt,
                appName = updated.metadata.name,
                appDescription = updated.metadata.description,
                appUrl = updated.metadata.url,
                appIcon = updated.metadata.icon,
            )
        )
    }

    override suspend fun deleteSessions(sessionIds: List<String>) = connectionsDao.delete(sessionIds)

    private fun DbConnection.toConnection(wallets: List<Wallet>): WalletConnection? {
        val wallet = wallets.firstOrNull { it.id.id == walletId } ?: return null
        return toDTO(wallet)
    }
}
