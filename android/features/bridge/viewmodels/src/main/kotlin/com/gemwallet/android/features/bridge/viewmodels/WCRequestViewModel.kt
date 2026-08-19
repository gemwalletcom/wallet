package com.gemwallet.android.features.bridge.viewmodels

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.blockchain.services.GemSignMessageOperator
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.data.repositories.bridge.ActiveWalletConnectRequest
import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.data.repositories.bridge.WalletConnectJsonRpcResponse
import com.gemwallet.android.data.repositories.bridge.WalletConnectSessionRequest
import com.gemwallet.android.data.repositories.bridge.WalletConnectVerifyContext
import com.gemwallet.android.data.repositories.bridge.fromWalletConnectChainId
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.bridge.viewmodels.model.BridgeRequestError
import com.gemwallet.android.features.bridge.viewmodels.model.WCRequest
import com.gemwallet.android.features.bridge.viewmodels.model.payload
import com.gemwallet.android.features.bridge.viewmodels.model.WalletConnectOriginVerifier
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.models.hasCriticalWarning
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletConnection
import com.wallet.core.primitives.WalletConnectionSession
import com.wallet.core.primitives.WalletConnectionSessionAppMetadata
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.WalletConnect
import uniffi.gemstone.WalletConnectAction
import javax.inject.Inject

@HiltViewModel
class WCRequestViewModel @Inject constructor(
    private val walletsRepository: WalletsRepository,
    private val bridgeRepository: BridgesRepository,
    private val passwordStore: PasswordStore,
    private val signMessageOperator: GemSignMessageOperator,
    private val simulationService: com.gemwallet.android.blockchain.services.WalletConnectSimulationService,
    private val getCurrentBlockExplorer: GetCurrentBlockExplorer,
    private val originVerifier: WalletConnectOriginVerifier,
    private val activeRequest: ActiveWalletConnectRequest,
) : ViewModel() {

    private val walletConnect = WalletConnect()
    private val state = MutableStateFlow(RequestViewModelState())
    private var requestJob: Job? = null
    val sceneState = state.map { it.toSceneState() }.stateIn(viewModelScope, SharingStarted.Eagerly, RequestSceneState.Loading)

    val buttonState = sceneState.map { scene ->
        val request = (scene as? RequestSceneState.Content)?.request as? WCRequest.SignMessage
        buttonState(
            enabled = request?.simulation?.warnings?.hasCriticalWarning() != true,
            loading = scene is RequestSceneState.Responding,
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Enabled)

    fun onRequest(
        sessionRequest: WalletConnectSessionRequest,
        verifyContext: WalletConnectVerifyContext,
        onNotify: (BridgeRequestError) -> Unit,
        onError: (String) -> Unit,
    ) {
        requestJob?.cancel()
        state.update { it.copy(sessionRequest = sessionRequest) }
        Log.d(TAG, "Resolving request method=${sessionRequest.request.method} chainId=${sessionRequest.chainId} id=${sessionRequest.request.id}")
        val job = viewModelScope.launch {
            try {
                val connection = bridgeRepository.getConnectionByTopic(sessionRequest.topic)
                if (connection == null) {
                    rejectRequest(sessionRequest)
                    return@launch
                }

                val appMetadata = connection.session.metadata
                if (originVerifier.verify(appMetadata.url, verifyContext).isScam) {
                    throw BridgeRequestError.MaliciousSession
                }
                val chainId = sessionRequest.chainId ?: throw BridgeRequestError.UnresolvedChainId
                val sessionDomain = appMetadata.url
                val action = walletConnect.parseRequest(
                    topic = sessionRequest.topic,
                    method = sessionRequest.request.method,
                    params = sessionRequest.request.params,
                    chainId = chainId,
                    domain = sessionDomain,
                )
                if (action is WalletConnectAction.Unsupported) {
                    respondMethodNotFound(sessionRequest, onError)
                    return@launch
                }
                if (action is WalletConnectAction.ChainOperation) {
                    handleChainOperation(action, sessionRequest, onError)
                    return@launch
                }
                if (action is WalletConnectAction.GetAccounts) {
                    handleGetAccounts(action, sessionRequest, connection, onError)
                    return@launch
                }

                val wallet = walletsRepository.getWallet(connection.wallet.id).firstOrNull()
                if (wallet == null) {
                    rejectRequest(sessionRequest)
                    return@launch
                }
                val chain = Chain.fromWalletConnectChainId(chainId)
                    ?: throw BridgeRequestError.ChainUnsupported

                validateChain(chain, connection.session)

                val account: Account = wallet.getAccount(chain) ?: throw BridgeRequestError.ChainUnsupported

                currentCoroutineContext().ensureActive()
                val request = buildRequest(action, sessionRequest, account, appMetadata, chain, sessionDomain)
                currentCoroutineContext().ensureActive()
                state.update {
                    it.copy(
                        request = request,
                        wallet = wallet,
                        chain = request.chain,
                    )
                }
            } catch (err: Throwable) {
                when (err) {
                    is CancellationException -> throw err
                    is BridgeRequestError -> handleRequestFailure(sessionRequest, err, onNotify)
                    else -> {
                        Log.e(TAG, "Request resolution failed method=${sessionRequest.request.method} chainId=${sessionRequest.chainId} id=${sessionRequest.request.id}", err)
                        onError(err.message ?: "Request failed")
                        rejectRequest(sessionRequest)
                    }
                }
            }
        }
        requestJob = job
        job.invokeOnCompletion {
            if (requestJob === job) {
                requestJob = null
            }
        }
    }

    private fun handleChainOperation(
        action: WalletConnectAction.ChainOperation,
        sessionRequest: WalletConnectSessionRequest,
        onError: (String) -> Unit,
    ) {
        when (action.operation) {
            uniffi.gemstone.WalletConnectChainOperation.AddChain,
            is uniffi.gemstone.WalletConnectChainOperation.SwitchChain -> {
                response(sessionRequest, "null", onError)
            }

            uniffi.gemstone.WalletConnectChainOperation.GetChainId -> {
                respondMethodNotFound(sessionRequest, onError)
            }
        }
    }

    private fun handleGetAccounts(
        action: WalletConnectAction.GetAccounts,
        sessionRequest: WalletConnectSessionRequest,
        connection: WalletConnection,
        onError: (String) -> Unit,
    ) {
        val chain = action.chain.toChain() ?: throw BridgeRequestError.ChainUnsupported
        validateChain(chain, connection.session)

        val accounts = connection.wallet.accounts
            .filter { it.chain == chain }
            .map { it.toGem() }
        response(sessionRequest, walletConnect.encodeGetAccounts(action.chain, accounts).payload(), onError)
    }

    private suspend fun buildRequest(
        action: WalletConnectAction,
        sessionRequest: WalletConnectSessionRequest,
        account: Account,
        appMetadata: WalletConnectionSessionAppMetadata,
        chain: Chain,
        sessionDomain: String,
    ): WCRequest = when (action) {
        is WalletConnectAction.SignMessage -> WCRequest.SignMessage(
            sessionRequest = sessionRequest,
            account = account,
            appMetadata = appMetadata,
            action = action,
            simulation = simulationService.simulateSignMessage(action.chain, action.signType, action.data, sessionDomain),
            explorerName = getCurrentBlockExplorer.getCurrentBlockExplorer(chain),
        )

        is WalletConnectAction.SendTransaction -> WCRequest.Transaction.SendTransaction(
            sessionRequest,
            account,
            appMetadata,
            action,
            simulationService.simulateSendTransaction(action.chain, action.transactionType, action.data),
        )

        is WalletConnectAction.SignTransaction -> WCRequest.Transaction.SignTransaction(
            sessionRequest,
            account,
            appMetadata,
            action,
            simulationService.simulateSendTransaction(action.chain, action.transactionType, action.data),
        )

        is WalletConnectAction.SignAllTransactions -> {
            val data = action.transactions.singleOrNull() ?: throw BridgeRequestError.MethodUnsupported
            WCRequest.Transaction.SignAllTransactions(
                sessionRequest = sessionRequest,
                account = account,
                appMetadata = appMetadata,
                transactionType = action.transactionType,
                data = data,
                simulation = simulationService.simulateSendTransaction(action.chain, action.transactionType, data),
            )
        }

        is WalletConnectAction.ChainOperation,
        is WalletConnectAction.GetAccounts -> error("Immediate WalletConnect responses must be handled before request resolution")
        is WalletConnectAction.Unsupported -> throw BridgeRequestError.MethodUnsupported
    }

    fun onTransactionResult(result: String, onError: (String) -> Unit) {
        viewModelScope.launch(Dispatchers.IO) {
            val snapshot = state.value
            if (snapshot.responseState == RequestResponseState.Responding) {
                return@launch
            }
            val request = snapshot.request as? WCRequest.Transaction ?: return@launch
            state.update { it.copy(responseState = RequestResponseState.Responding) }
            val response = try {
                request.execute(result)
            } catch (err: Throwable) {
                Log.e(TAG, "Transaction response encoding failed id=${request.requestId}", err)
                state.update { it.copy(responseState = RequestResponseState.Idle) }
                onError(err.message ?: "Request failed")
                rejectRequest(request.sessionRequest)
                return@launch
            }
            response(request.sessionRequest, response, onError)
        }
    }

    fun onSign(onError: (String) -> Unit) {
        val snapshot = state.value
        if (snapshot.responseState == RequestResponseState.Responding) {
            return
        }
        val request = (snapshot.request as? WCRequest.SignMessage) ?: return
        val wallet = snapshot.wallet ?: return
        val chain = snapshot.chain ?: return

        state.update { it.copy(responseState = RequestResponseState.Responding) }
        viewModelScope.launch(Dispatchers.IO) {
            val password = passwordStore.getPassword(wallet.id.id)
            val sign = try {
                request.execute(signMessageOperator, wallet, password)
            } catch (err: Throwable) {
                Log.e(TAG, "Sign message response encoding failed id=${request.requestId}", err)
                state.update { it.copy(responseState = RequestResponseState.Idle) }
                onError(err.message ?: "Sign failed")
                rejectRequest(request.sessionRequest)
                return@launch
            }
            response(request.sessionRequest, sign, onError)
        }
    }

    private fun response(sessionRequest: WalletConnectSessionRequest, payload: String, onError: (String) -> Unit) {
        bridgeRepository.respondSessionRequest(
            topic = sessionRequest.topic,
            id = sessionRequest.request.id,
            response = WalletConnectJsonRpcResponse.Result(payload),
            onSuccess = { activeRequest.finish(sessionRequest) },
            onError = { error ->
                state.update { it.copy(responseState = RequestResponseState.Idle) }
                onError(error.ifBlank { "Request failed" })
            },
        )
    }

    private fun respondError(sessionRequest: WalletConnectSessionRequest, code: Int, message: String, onError: (String) -> Unit) {
        bridgeRepository.respondSessionRequest(
            topic = sessionRequest.topic,
            id = sessionRequest.request.id,
            response = WalletConnectJsonRpcResponse.Error(code, message),
            onSuccess = { activeRequest.finish(sessionRequest) },
            onError = { error -> onError(error.ifBlank { "Request failed" }) },
        )
    }

    private fun respondMethodNotFound(sessionRequest: WalletConnectSessionRequest, onError: (String) -> Unit) {
        respondError(
            sessionRequest = sessionRequest,
            code = -32601,
            message = "Method not found",
            onError = onError,
        )
    }

    fun onReject() {
        if (state.value.responseState == RequestResponseState.Responding) {
            return
        }
        requestJob?.cancel()
        val sessionRequest = state.value.sessionRequest ?: return
        rejectRequest(sessionRequest)
    }

    private fun handleRequestFailure(
        sessionRequest: WalletConnectSessionRequest,
        error: BridgeRequestError,
        onNotify: (BridgeRequestError) -> Unit
    ) {
        Log.e(TAG, "Request rejected method=${sessionRequest.request.method} chainId=${sessionRequest.chainId} id=${sessionRequest.request.id}: ${error.message}")
        if (error is BridgeRequestError.MaliciousSession) {
            onNotify(error)
        }
        rejectRequest(sessionRequest)
    }

    private fun rejectRequest(sessionRequest: WalletConnectSessionRequest) {
        bridgeRepository.respondSessionRequest(
            topic = sessionRequest.topic,
            id = sessionRequest.request.id,
            response = WalletConnectJsonRpcResponse.Error(
                code = 4001,
                message = "User rejected the request",
            ),
            onSuccess = { activeRequest.finish(sessionRequest) },
            onError = { error -> Log.e(TAG, "Request rejection failed id=${sessionRequest.request.id}: $error") },
        )
    }

    fun reset() {
        requestJob?.cancel()
        requestJob = null
        state.update { RequestViewModelState() }
    }

    private fun validateChain(chain: Chain, session: WalletConnectionSession) {
        if (!session.chains.contains(chain)) {
            throw BridgeRequestError.UnresolvedChainId
        }
    }

    private companion object {
        const val TAG = "WalletConnect"
    }
}

private data class RequestViewModelState(
    val sessionRequest: WalletConnectSessionRequest? = null,
    val wallet: com.wallet.core.primitives.Wallet? = null,
    val request: WCRequest? = null,
    val chain: Chain? = null,
    val responseState: RequestResponseState = RequestResponseState.Idle,
) {
    fun toSceneState(): RequestSceneState {
        if (request == null) {
            return RequestSceneState.Loading
        }
        wallet ?: return RequestSceneState.Loading

        val requestState = RequestSceneState.Request(walletName = wallet.name, request = request)
        return when (responseState) {
            RequestResponseState.Idle -> requestState
            RequestResponseState.Responding -> RequestSceneState.Responding(requestState)
        }
    }
}

private enum class RequestResponseState {
    Idle,
    Responding,
}

sealed interface RequestSceneState {

    data object Loading : RequestSceneState

    sealed interface Content : RequestSceneState {
        val walletName: String
        val request: WCRequest
    }

    class Request(
        override val walletName: String,
        override val request: WCRequest,
    ) : Content

    class Responding(
        private val requestState: Request,
    ) : Content {
        override val walletName: String get() = requestState.walletName
        override val request: WCRequest get() = requestState.request
    }
}
