package com.gemwallet.android.data.repositories.bridge

import androidx.core.net.toUri
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.ConnectionsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.canSign
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.WalletConnection
import com.wallet.core.primitives.Wallet as GemWallet
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.WalletConnect

@OptIn(ExperimentalCoroutinesApi::class)
class BridgesRepository(
    private val walletsRepository: WalletsRepository,
    private val connectionsDao: ConnectionsDao,
    private val walletConnectClient: WalletConnectClient,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO + SupervisorJob()),
) {

    val isWalletConnectEnabled: Boolean = walletConnectClient.isEnabled

    private val pendingEvents = MutableSharedFlow<WalletConnectEvent>(extraBufferCapacity = 16)
    private val isWalletConnectInit = MutableStateFlow(false)
    val bridgeEvents = isWalletConnectInit.flatMapLatest {
        if (it) {
            merge(walletConnectClient.events, pendingEvents)
        } else {
            emptyFlow()
        }
    }

    init {
        scope.launch(Dispatchers.IO) {
            if ((getConnections().firstOrNull() ?: emptyList()).isNotEmpty()) {
                initWalletConnect()
                sync()
                pingActiveSessions()
                handlePendingRequests()
            }
        }
        scope.launch(Dispatchers.IO) {
            bridgeEvents.collect { event ->
                when (event) {
                    is WalletConnectEvent.SessionSettled -> addConnection(event.session)
                    is WalletConnectEvent.SessionDeleted -> deleteConnection(event.topic)
                    else -> Unit
                }
            }
        }
    }

    private fun initWalletConnect(onSuccess: () -> Unit = {}, onError: (String) -> Unit = {}) {
        if (isWalletConnectInit.value) {
            onSuccess()
            return
        }
        walletConnectClient.initialize(
            onSuccess = {
                isWalletConnectInit.update { true }
                onSuccess()
            },
            onError = onError,
        )
    }

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

    private suspend fun sync() {
        val local = getConnections().firstOrNull() ?: emptyList()
        val sessions = activeSessions()
        val unknownSessions = local.filter { local -> !sessions.any { local.session.sessionId == it.topic } }
        if (unknownSessions.isNotEmpty()) {
            connectionsDao.deleteAll(unknownSessions.map { it.toRecord() })
        }
        val localSessionIds = local.map { it.session.sessionId }.toSet()
        sessions
            .filter { it.topic in localSessionIds }
            .forEach { updateConnection(it) }
    }

    private fun handlePendingRequests() {
        for (session in activeSessions()) {
            val request = walletConnectClient.pendingSessionRequests(session.topic).firstOrNull() ?: continue
            val verifyContext = walletConnectClient.verifyContext(request.request.id) ?: continue
            pendingEvents.tryEmit(WalletConnectEvent.SessionRequest(request, verifyContext))
        }
    }

    private fun pingActiveSessions() {
        for (session in activeSessions()) {
            walletConnectClient.pingSession(session.topic)
        }
    }

    private fun activeSessions(): List<WalletConnectSession> =
        runCatching { walletConnectClient.activeSessions().filter { it.metadata != null } }
            .getOrDefault(emptyList())

    suspend fun disconnect(id: String, onSuccess: () -> Unit = {}, onError: (String) -> Unit = {}) {
        val connection = getConnections().firstOrNull()?.firstOrNull { it.session.id == id } ?: return
        connectionsDao.delete(id)
        val activeSession = activeSessions().firstOrNull { it.topic == connection.session.sessionId }
        if (activeSession != null) {
            walletConnectClient.disconnectSession(activeSession.topic, onSuccess = {}, onError = {})
        }
        onSuccess()
    }

    fun addPairing(uri: String, onSuccess: () -> Unit = {}, onError: (String) -> Unit = {}) {
        initWalletConnect(
            onSuccess = {
                try {
                    walletConnectClient.pair(
                        uri = uri,
                        onSuccess = { onSuccess() },
                        onError = { onError(it.ifBlank { "Pair to ${uri.toUri().host} fail" }) },
                    )
                } catch (err: Throwable) {
                    onError("Wallet Connect unavailable: ${err.message}")
                }
            },
            onError = { onError(it.ifBlank { "Wallet Connect unavailable" }) },
        )
    }

    fun approveConnection(
        wallet: GemWallet,
        proposal: WalletConnectSessionProposal,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        val supportedNamespaces = getSupportedNamespaces(wallet)
        val sessionNamespaces = walletConnectClient.generateApprovedNamespaces(
            proposal = proposal,
            supportedNamespaces = supportedNamespaces,
        )
        val sessionProperties = WalletConnect().configSessionProperties(
            properties = proposal.properties ?: emptyMap(),
            caip2Chains = sessionNamespaces.values.flatMap { it.chains.orEmpty() },
        )
        val activeBefore = activeSessions().map { it.topic }.toSet()

        walletConnectClient.approveSession(
            proposal = proposal,
            namespaces = sessionNamespaces,
            properties = sessionProperties,
            onSuccess = {
                persistNewSessions(wallet, activeBefore, "Connection failed", onSuccess, onError)
            },
            onError = onError,
        )
    }

    fun rejectConnection(
        proposal: WalletConnectSessionProposal,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        walletConnectClient.rejectSession(proposal, onSuccess, onError)
    }

    fun approveAuthentication(
        request: WalletConnectAuthenticationRequest,
        auths: List<WalletConnectAuthObject>,
        wallet: GemWallet,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        val activeBefore = activeSessions().map { it.topic }.toSet()
        walletConnectClient.approveAuthentication(
            request = request,
            auths = auths,
            onSuccess = {
                persistNewSessions(wallet, activeBefore, "Authentication failed", onSuccess, onError)
            },
            onError = onError,
        )
    }

    fun rejectAuthentication(
        request: WalletConnectAuthenticationRequest,
        onSuccess: () -> Unit = {},
        onError: (String) -> Unit = {},
    ) {
        walletConnectClient.rejectAuthentication(request, onSuccess, onError)
    }

    fun respondSessionRequest(
        topic: String,
        id: Long,
        response: WalletConnectJsonRpcResponse,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        walletConnectClient.respondSessionRequest(topic, id, response, onSuccess, onError)
    }

    fun generateAuthPayloadParams(
        payloadParams: WalletConnectAuthPayloadParams,
        supportedChains: List<String>,
        supportedMethods: List<String>,
    ): WalletConnectAuthPayloadParams {
        return walletConnectClient.generateAuthPayloadParams(payloadParams, supportedChains, supportedMethods)
    }

    fun formatAuthMessage(payloadParams: WalletConnectAuthPayloadParams, issuer: String): String {
        return walletConnectClient.formatAuthMessage(payloadParams, issuer)
    }

    fun generateAuthObject(
        payloadParams: WalletConnectAuthPayloadParams,
        issuer: String,
        signature: String,
    ): WalletConnectAuthObject {
        return walletConnectClient.generateAuthObject(payloadParams, issuer, signature)
    }

    private suspend fun addConnection(session: WalletConnectSession) {
        if (connectionsDao.getBySessionId(session.topic) != null) {
            updateConnection(session)
            return
        }
        val wallet = walletForSession(session) ?: return
        addConnection(session, wallet)
    }

    private suspend fun addConnection(
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

    private suspend fun updateConnection(session: WalletConnectSession) {
        val record = connectionsDao.getBySessionId(session.topic) ?: return
        connectionsDao.insert(
            session.toConnectionRecord(
                walletId = record.walletId,
                createdAt = record.createdAt,
            )
        )
    }

    private suspend fun deleteConnection(topic: String) {
        connectionsDao.delete(topic)
    }

    private fun persistNewSessions(
        wallet: GemWallet,
        activeBefore: Set<String>,
        failureMessage: String,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        scope.launch(Dispatchers.IO) {
            runCatching {
                addNewSessions(wallet, activeBefore)
            }.onSuccess {
                onSuccess()
            }.onFailure { error ->
                onError(error.message ?: failureMessage)
            }
        }
    }

    private suspend fun addNewSessions(
        wallet: GemWallet,
        activeBefore: Set<String>,
    ) {
        val localSessionIds = connectionsDao.getAll().firstOrNull()
            ?.map { it.sessionId }
            ?.toSet()
            ?: emptySet()
        activeSessions()
            .filter { it.topic !in localSessionIds }
            .filter { it.topic !in activeBefore }
            .filter { it.belongsTo(wallet) }
            .forEach { addConnection(it, wallet) }
    }

    private suspend fun walletForSession(session: WalletConnectSession): GemWallet? {
        val sessionAccounts = session.accounts()
        if (sessionAccounts.isEmpty()) return null
        val wallets = walletsRepository.getAll().firstOrNull() ?: return null
        return wallets.firstOrNull { wallet ->
            wallet.type.canSign && sessionAccounts.belongsTo(wallet)
        }
    }

    private fun getSupportedNamespaces(wallet: GemWallet): Map<String, WalletConnectSessionNamespace> {
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
