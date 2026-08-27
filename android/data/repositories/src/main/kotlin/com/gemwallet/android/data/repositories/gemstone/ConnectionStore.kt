package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.repositories.bridge.ConnectionsRepository
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.WalletConnection
import com.wallet.core.primitives.WalletConnectionSession
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemConnectionStore

class GemstoneConnectionStore(
    private val connectionsRepository: ConnectionsRepository,
) : GemConnectionStore {
    override suspend fun getConnection(sessionId: String): String? =
        connectionsRepository.getConnectionByTopic(sessionId)?.toJson()

    override suspend fun getSessions(): List<String> =
        connectionsRepository.getConnections().firstOrNull().orEmpty().map { it.session.toJson() }

    override suspend fun addConnection(connection: String) =
        connectionsRepository.addConnection(connection.decodeJson<WalletConnection>())

    override suspend fun updateSession(session: String) =
        connectionsRepository.updateSession(session.decodeJson<WalletConnectionSession>())

    override suspend fun deleteSessions(sessionIds: List<String>) =
        connectionsRepository.deleteSessions(sessionIds)
}
