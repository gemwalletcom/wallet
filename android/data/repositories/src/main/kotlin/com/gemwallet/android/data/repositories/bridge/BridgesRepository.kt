package com.gemwallet.android.data.repositories.bridge

import android.app.Application
import android.content.Context
import androidx.core.net.toUri
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.ConnectionsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.canSign
import com.reown.android.Core
import com.reown.android.CoreClient
import com.reown.android.relay.ConnectionType
import com.reown.walletkit.client.Wallet
import com.reown.walletkit.client.WalletKit
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.WalletConnection
import com.wallet.core.primitives.Wallet as GemWallet
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.WalletConnect

@OptIn(ExperimentalCoroutinesApi::class)
class BridgesRepository(
    private val context: Context,
    private val walletsRepository: WalletsRepository,
    private val connectionsDao: ConnectionsDao,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO + SupervisorJob()),
) {

    private val isWalletConnectInit = MutableStateFlow(false)
    val bridgeEvents = isWalletConnectInit.flatMapLatest {
        if (it) {
            WalletConnectDelegate.walletEvents
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
                when (val model = event.model) {
                    is Wallet.Model.SettledSessionResponse.Result -> addConnection(model.session)
                    is Wallet.Model.SessionDelete.Success -> deleteConnection(model.topic)
                    else -> Unit
                }
            }
        }
    }

    private fun initWalletConnect(onSuccess: () -> Unit = {}) {
        if (isWalletConnectInit.value) {
            onSuccess()
            return
        }

        val projectId = "3bc07cd7179d11ea65335fb9377702b6"
        val connectionType = ConnectionType.AUTOMATIC
        val metaData = Core.Model.AppMetaData(
            name = "Gem Wallet",
            description = "Gem Web3 Wallet",
            url = "https://gemwallet.com",
            icons = listOf("https://gemwallet.com/images/gem-logo-256x256.png"),
            redirect = "gem://wc/"
        )
        CoreClient.initialize(
            application = context as Application,
            projectId = projectId,
            metaData = metaData,
            connectionType = connectionType,
            telemetryEnabled = false,
        ) {
        }
        val initParams = Wallet.Params.Init(core = CoreClient)
        WalletKit.initialize(
            initParams,
            {
                WalletConnectDelegate.bind()
                isWalletConnectInit.update { true }
                onSuccess()
            }
        ) { _ -> }
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
            val request = WalletKit.getPendingListOfSessionRequests(session.topic)
                .firstOrNull() ?: continue
            val verifyContext = WalletKit.getVerifyContext(request.request.id) ?: continue
            WalletConnectDelegate.onSessionRequest(request, verifyContext)
        }
    }

    private fun pingActiveSessions() {
        for (session in activeSessions()) {
            WalletKit.pingSession(Wallet.Params.Ping(session.topic), null)
        }
    }

    private fun activeSessions(): List<Wallet.Model.Session> =
        runCatching { WalletKit.getListOfActiveSessions().filter { it.metaData != null } }
            .getOrDefault(emptyList())

    suspend fun disconnect(id: String, onSuccess: () -> Unit = {}, onError: (String) -> Unit = {}) {
        val connection = getConnections().firstOrNull()?.firstOrNull { it.session.id == id } ?: return
        connectionsDao.delete(id)
        val activeSession = activeSessions().firstOrNull { it.topic == connection.session.sessionId }
        if (activeSession != null) {
            WalletKit.disconnectSession(
                params = Wallet.Params.SessionDisconnect(activeSession.topic),
                onSuccess = {},
                onError = {},
            )
        }
        onSuccess()
    }

    fun addPairing(uri: String, onSuccess: () -> Unit = {}, onError: (String) -> Unit = {}) {
        initWalletConnect(
            {
                try {
                    WalletKit.pair(
                        params = Wallet.Params.Pair(uri),
                        onSuccess = { onSuccess() },
                        onError = {
                            onError(it.throwable.message ?: "Pair to ${uri.toUri().host} fail")
                        }
                    )
                } catch (err: Throwable) {
                    onError("Wallet Connect unavailable: ${err.message}")
                }
            }
        )
    }

    fun approveConnection(
        wallet: GemWallet,
        proposal: Wallet.Model.SessionProposal,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        val sessionProposal = getPendingSessionProposal(proposal)
        if (sessionProposal == null) {
            onSuccess()
            return
        }
        val supportedNamespaces = getSupportedNamespaces(wallet)
        val sessionNamespaces = WalletKit.generateApprovedNamespaces(
            sessionProposal = sessionProposal,
            supportedNamespaces = supportedNamespaces
        )
        val sessionProperties = WalletConnect().configSessionProperties(
            properties = proposal.properties ?: emptyMap(),
            caip2Chains = sessionNamespaces.values.flatMap { it.chains.orEmpty() },
        )
        val approveProposal = Wallet.Params.SessionApprove(
            proposerPublicKey = sessionProposal.proposerPublicKey,
            namespaces = sessionNamespaces,
            properties = sessionProperties,
        )
        val activeBefore = activeSessions().map { it.topic }.toSet()

        WalletKit.approveSession(
            params = approveProposal,
            onError = { error -> onError(error.throwable.message ?: "Unknown error") },
            onSuccess = {
                persistNewSessions(wallet, activeBefore, "Connection failed", onSuccess, onError)
            },
        )
    }

    fun rejectConnection(
        proposal: Wallet.Model.SessionProposal,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        val sessionProposal = getPendingSessionProposal(proposal)
        if (sessionProposal == null) {
            onSuccess()
            return
        }

        WalletKit.rejectSession(
            params = Wallet.Params.SessionReject(
                proposerPublicKey = sessionProposal.proposerPublicKey,
                reason = "Reject Session"
            ),
            onSuccess = {
                onSuccess()
            },
            onError = {
                onError(it.throwable.message ?: "")
            },
        )
    }

    fun approveAuthentication(
        request: Wallet.Model.SessionAuthenticate,
        auths: List<Wallet.Model.Cacao>,
        wallet: GemWallet,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        val activeBefore = activeSessions().map { it.topic }.toSet()
        WalletKit.approveSessionAuthenticate(
            params = Wallet.Params.ApproveSessionAuthenticate(
                id = request.id,
                auths = auths,
            ),
            onSuccess = {
                persistNewSessions(wallet, activeBefore, "Authentication failed", onSuccess, onError)
            },
            onError = { error ->
                onError(error.throwable.message ?: "Authentication failed")
            },
        )
    }

    fun rejectAuthentication(
        request: Wallet.Model.SessionAuthenticate,
        onSuccess: () -> Unit = {},
        onError: (String) -> Unit = {},
    ) {
        WalletKit.rejectSessionAuthenticate(
            params = Wallet.Params.RejectSessionAuthenticate(
                id = request.id,
                reason = "Reject Session Authentication",
            ),
            onSuccess = {
                onSuccess()
            },
            onError = {
                onError(it.throwable.message ?: "")
            },
        )
    }

    private fun getPendingSessionProposal(proposal: Wallet.Model.SessionProposal): Wallet.Model.SessionProposal? {
        val proposalPublicKey = proposal.proposerPublicKey
        return WalletKit.getSessionProposals().firstOrNull { it.proposerPublicKey == proposalPublicKey }
    }

    private suspend fun addConnection(session: Wallet.Model.Session) {
        if (connectionsDao.getBySessionId(session.topic) != null) {
            updateConnection(session)
            return
        }
        val wallet = walletForSession(session)
            ?: return
        addConnection(session, wallet)
    }

    private suspend fun addConnection(
        session: Wallet.Model.Session,
        wallet: GemWallet,
    ) {
        connectionsDao.insert(
            session.toConnectionRecord(
                walletId = wallet.id.id,
                createdAt = System.currentTimeMillis(),
            )
        )
    }

    private suspend fun updateConnection(session: Wallet.Model.Session) {
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

    private suspend fun walletForSession(session: Wallet.Model.Session): GemWallet? {
        val sessionAccounts = session.accounts()
        if (sessionAccounts.isEmpty()) return null
        val wallets = walletsRepository.getAll().firstOrNull() ?: return null
        return wallets.firstOrNull { wallet ->
            wallet.type.canSign && sessionAccounts.belongsTo(wallet)
        }
    }

    private fun getSupportedNamespaces(wallet: GemWallet): Map<String, Wallet.Model.Namespace.Session> {
        return wallet.accounts
            .mapNotNull { it.toSupportedAccount() }
            .groupBy { it.namespace }
            .map { (namespace, accounts) ->
                namespace.string to Wallet.Model.Namespace.Session(
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
