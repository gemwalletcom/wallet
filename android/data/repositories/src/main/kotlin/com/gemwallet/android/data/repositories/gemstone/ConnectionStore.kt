package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.ConnectionsDao
import com.gemwallet.android.data.service.store.database.entities.DbConnection
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toSession
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletConnection
import com.wallet.core.primitives.WalletConnectionSession
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemConnectionStore

@OptIn(ExperimentalCoroutinesApi::class)
class GemstoneConnectionStore(
    private val walletsRepository: WalletsRepository,
    private val connectionsDao: ConnectionsDao,
) : GemConnectionStore {

    fun observeConnections(): Flow<List<WalletConnection>> = walletsRepository.getAll().flatMapLatest { wallets ->
        connectionsDao.getAll().map { records -> records.mapNotNull { it.toConnection(wallets) } }
    }

    fun observeConnection(connectionId: String): Flow<WalletConnection?> = walletsRepository.getAll().flatMapLatest { wallets ->
        connectionsDao.getConnection(connectionId).map { it?.toConnection(wallets) }
    }

    suspend fun getConnectionBySessionId(sessionId: String): WalletConnection? {
        val record = connectionsDao.getBySessionId(sessionId) ?: return null
        return record.toConnection(walletsRepository.getAll().firstOrNull().orEmpty())
    }

    override suspend fun getConnection(sessionId: String): String? = getConnectionBySessionId(sessionId)?.toJson()

    override suspend fun getSessions(): List<String> = connectionsDao.getConnections().map { it.toSession().toJson() }

    override suspend fun addConnection(connection: String) = connectionsDao.insert(connection.decodeJson<WalletConnection>().toRecord())

    override suspend fun updateSession(session: String) {
        val updated = session.decodeJson<WalletConnectionSession>()
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
