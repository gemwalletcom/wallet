package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.ConnectionsDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.database.entities.toSession
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.WalletConnection
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemConnectionStore

class GemstoneConnectionStore(private val walletStore: GemstoneWalletStore, private val connectionsDao: ConnectionsDao) : GemConnectionStore {

    suspend fun getConnectionBySessionId(sessionId: String): WalletConnection? {
        val record = connectionsDao.getBySessionId(sessionId) ?: return null
        return record.toDTO(walletStore.observeWallets().firstOrNull().orEmpty())
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
            ),
        )
    }

    override suspend fun deleteSessions(sessionIds: List<String>) = connectionsDao.delete(sessionIds)
}
