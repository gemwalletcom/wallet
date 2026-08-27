package com.gemwallet.android.data.repositories.bridge

import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.ConnectionsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.wallet.core.primitives.WalletConnection
import com.wallet.core.primitives.WalletConnectionSession
import com.wallet.core.primitives.Wallet as GemWallet
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map

@OptIn(ExperimentalCoroutinesApi::class)
class ConnectionsRepository(
    private val walletsRepository: WalletsRepository,
    private val connectionsDao: ConnectionsDao,
) {

    fun getConnections(): Flow<List<WalletConnection>> {
        return walletsRepository.getAll().flatMapLatest { wallets ->
            connectionsDao.getAll().map { items ->
                items.mapNotNull { room ->
                    val wallet = wallets.firstOrNull { it.id.id == room.walletId } ?: return@mapNotNull null
                    room.toDTO(wallet)
                }
            }
        }
    }

    suspend fun getConnectionByTopic(topic: String): WalletConnection? {
        val record = connectionsDao.getBySessionId(topic) ?: return null
        val wallet = walletsRepository.getAll().firstOrNull()
            ?.firstOrNull { it.id.id == record.walletId }
            ?: return null
        return record.toDTO(wallet)
    }

    fun getConnection(connectionId: String): Flow<WalletConnection?> {
        return walletsRepository.getAll().flatMapLatest { wallets ->
            connectionsDao.getConnection(connectionId).map { room ->
                val wallet = wallets.firstOrNull { it.id.id == room?.walletId } ?: return@map null
                room?.toDTO(wallet)
            }
        }
    }

    suspend fun disconnect(id: String): WalletConnection? {
        val connection = getConnections().firstOrNull()?.firstOrNull { it.session.id == id } ?: return null
        connectionsDao.delete(id)
        return connection
    }

    suspend fun deleteConnection(topic: String) {
        connectionsDao.delete(topic)
    }

    suspend fun addConnection(connection: WalletConnection) {
        connectionsDao.insert(connection.toRecord())
    }

    suspend fun updateSession(session: WalletConnectionSession) {
        val record = connectionsDao.getBySessionId(session.sessionId) ?: return
        connectionsDao.insert(
            record.copy(
                state = session.state,
                chains = session.chains,
                expireAt = session.expireAt,
                appName = session.metadata.name,
                appDescription = session.metadata.description,
                appUrl = session.metadata.url,
                appIcon = session.metadata.icon,
            )
        )
    }

    suspend fun deleteSessions(sessionIds: List<String>) {
        sessionIds.forEach { connectionsDao.delete(it) }
    }
}
