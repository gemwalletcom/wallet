package com.gemwallet.android.data.repositories.bridge

import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.ConnectionsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.canSign
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.WalletConnection
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

    suspend fun sync(sessions: List<WalletConnectSession>?) {
        if (sessions == null) return
        val local = getConnections().firstOrNull() ?: emptyList()
        val unknownSessions = local.filter { local -> !sessions.any { local.session.sessionId == it.topic } }
        if (unknownSessions.isNotEmpty()) {
            connectionsDao.deleteAll(unknownSessions.map { it.toRecord() })
        }
        val localSessionIds = local.map { it.session.sessionId }.toSet()
        sessions
            .filter { it.topic in localSessionIds }
            .forEach { updateConnection(it) }
    }

    suspend fun disconnect(id: String): WalletConnection? {
        val connection = getConnections().firstOrNull()?.firstOrNull { it.session.id == id } ?: return null
        connectionsDao.delete(id)
        return connection
    }

    suspend fun addConnection(session: WalletConnectSession) {
        if (connectionsDao.getBySessionId(session.topic) != null) {
            updateConnection(session)
            return
        }
        val wallet = walletForSession(session) ?: return
        addConnection(session, wallet)
    }

    suspend fun addConnection(
        session: WalletConnectSession,
        wallet: GemWallet,
    ) {
        connectionsDao.insert(
            session.toConnectionRecord(
                walletId = wallet.id.id,
                createdAt = System.currentTimeMillis(),
            )
        )
    }

    suspend fun updateConnection(session: WalletConnectSession) {
        val record = connectionsDao.getBySessionId(session.topic) ?: return
        connectionsDao.insert(
            session.toConnectionRecord(
                walletId = record.walletId,
                createdAt = record.createdAt,
            )
        )
    }

    suspend fun deleteConnection(topic: String) {
        connectionsDao.delete(topic)
    }

    suspend fun addNewSessions(
        wallet: GemWallet,
        sessions: List<WalletConnectSession>,
        activeBefore: Set<String>,
    ) {
        val localSessionIds = connectionsDao.getAll().firstOrNull()
            ?.map { it.sessionId }
            ?.toSet()
            ?: emptySet()
        sessions
            .filter { it.topic !in localSessionIds }
            .filter { it.topic !in activeBefore }
            .filter { it.belongsTo(wallet) }
            .forEach { addConnection(it, wallet) }
    }

    suspend fun walletForSession(session: WalletConnectSession): GemWallet? {
        val sessionAccounts = session.accounts()
        if (sessionAccounts.isEmpty()) return null
        val wallets = walletsRepository.getAll().firstOrNull() ?: return null
        return wallets.firstOrNull { wallet ->
            wallet.type.canSign && sessionAccounts.belongsTo(wallet)
        }
    }

    fun getSupportedNamespaces(wallet: GemWallet): Map<String, WalletConnectSessionNamespace> {
        return wallet.accounts
            .mapNotNull { it.toSupportedAccount() }
            .groupBy { it.namespace }
            .map { (namespace, accounts) ->
                namespace.string to WalletConnectSessionNamespace(
                    chains = accounts.map { it.chainId },
                    methods = namespace.methodIds,
                    events = namespace.eventIds,
                    accounts = accounts.map { it.accountId },
                )
            }
            .toMap()
    }
}

private data class SupportedAccount(
    val account: Account,
    val namespace: ChainNamespace,
    val reference: String,
) {
    val chainId: String get() = "${namespace.string}:$reference"
    val accountId: String get() = "$chainId:${account.address}"
}

private fun Account.toSupportedAccount(): SupportedAccount? {
    val namespace = chain.walletConnectNamespace() ?: return null
    val reference = chain.walletConnectReference() ?: return null
    return SupportedAccount(this, namespace, reference)
}
