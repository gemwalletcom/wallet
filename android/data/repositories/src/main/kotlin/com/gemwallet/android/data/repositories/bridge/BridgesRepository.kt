package com.gemwallet.android.data.repositories.bridge

import android.app.Application
import android.content.Context
import androidx.core.net.toUri
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.ConnectionsDao
import com.gemwallet.android.data.service.store.database.entities.DbConnection
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.canSign
import com.gemwallet.android.ext.walletConnectAppName
import com.gemwallet.android.ext.walletConnectIcon
import com.reown.android.Core
import com.reown.android.CoreClient
import com.reown.android.relay.ConnectionType
import com.reown.walletkit.client.Wallet
import com.reown.walletkit.client.WalletKit
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletConnection
import com.wallet.core.primitives.WalletConnectionEvents
import com.wallet.core.primitives.WalletConnectionState
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
                    is Wallet.Model.SessionDelete.Success -> sync()
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
            ?.firstOrNull { it.id.id == record.walletId } ?: return null
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
        val activeSession = activeSessions().firstOrNull { it.topic == connection.session.sessionId }
        if (activeSession != null) {
            WalletKit.disconnectSession(
                params = Wallet.Params.SessionDisconnect(activeSession.topic),
                onSuccess = {},
                onError = {},
            )
        }
        connectionsDao.delete(id)
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
        wallet: com.wallet.core.primitives.Wallet,
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
        val approveProposal = Wallet.Params.SessionApprove(
            proposerPublicKey = sessionProposal.proposerPublicKey,
            namespaces = sessionNamespaces,
            properties = proposal.properties ?: emptyMap(),
        )

        WalletKit.approveSession(
            params = approveProposal,
            onError = { error -> onError(error.throwable.message ?: "Unknown error") },
            onSuccess = { onSuccess() },
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
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        WalletKit.approveSessionAuthenticate(
            params = Wallet.Params.ApproveSessionAuthenticate(
                id = request.id,
                auths = auths,
            ),
            onSuccess = { onSuccess() },
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
        val wallet = resolveWallet(session) ?: return
        val chains = session.namespaces.values
            .flatMap { it.chains ?: emptyList() }
            .mapNotNull { Chain.getNamespace(it) }
        connectionsDao.insert(
            DbConnection(
                id = session.topic,
                walletId = wallet.id.id,
                sessionId = session.topic,
                state = WalletConnectionState.Active,
                chains = chains,
                createdAt = System.currentTimeMillis(),
                expireAt = session.expiry * 1000L,
                appName = walletConnectAppName(session.metaData?.name, session.metaData?.url),
                appDescription = session.metaData?.description ?: "",
                appUrl = session.metaData?.url ?: "",
                appIcon = session.metaData?.icons.walletConnectIcon(),
                redirectNative = session.redirect,
                redirectUniversal = session.redirect,
            )
        )
    }

    private suspend fun resolveWallet(session: Wallet.Model.Session): com.wallet.core.primitives.Wallet? {
        val addresses = session.namespaces.values
            .flatMap { it.accounts }
            .mapNotNull { it.split(":").getOrNull(2) }
            .toSet()
        if (addresses.isEmpty()) return null
        val wallets = walletsRepository.getAll().firstOrNull() ?: return null
        return wallets.firstOrNull { wallet ->
            wallet.type.canSign && wallet.accounts.any { it.address in addresses }
        }
    }

    private fun getSupportedNamespaces(wallet: com.wallet.core.primitives.Wallet): Map<String, Wallet.Model.Namespace.Session> {
        return wallet.accounts
            .mapNotNull { WalletConnectAccount.create(it) }
            .groupBy { it.namespace }
            .mapValues { chain ->
                Wallet.Model.Namespace.Session(
                    chains = chain.value.map { "${it.namespace}:${it.reference}" },
                    methods = chain.value.map { it.methods }.toSet().flatten().toList(),
                    events = if (chain.key == Chain.Solana.string) emptyList() else listOf(
                        WalletConnectionEvents.Connect.string,
                        WalletConnectionEvents.Disconnect.string,
                        WalletConnectionEvents.ChainChanged.string,
                        WalletConnectionEvents.AccountsChanged.string
                    ),
                    accounts = chain.value.map { "${it.namespace}:${it.reference}:${it.address}" },
                )
            }
    }

}
