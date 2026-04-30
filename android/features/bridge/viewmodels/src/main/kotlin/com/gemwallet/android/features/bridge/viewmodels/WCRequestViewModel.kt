package com.gemwallet.android.features.bridge.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.blockchain.operators.LoadPrivateKeyOperator
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.data.repositories.bridge.getNamespace
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.features.bridge.viewmodels.model.BridgeRequestError
import com.gemwallet.android.features.bridge.viewmodels.model.WCRequest
import com.gemwallet.android.features.bridge.viewmodels.model.map
import com.reown.walletkit.client.Wallet
import com.reown.walletkit.client.WalletKit
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletConnectionSession
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
import com.wallet.core.primitives.SimulationResult
import uniffi.gemstone.WalletConnect
import uniffi.gemstone.WalletConnectAction
import uniffi.gemstone.WalletConnectionVerificationStatus
import java.util.Arrays
import javax.inject.Inject

@HiltViewModel
class WCRequestViewModel @Inject constructor(
    private val walletsRepository: WalletsRepository,
    private val bridgeRepository: BridgesRepository,
    private val passwordStore: PasswordStore,
    private val loadPrivateKeyOperator: LoadPrivateKeyOperator,
    private val simulationService: com.gemwallet.android.blockchain.services.WalletConnectSimulationService,
    private val getCurrentBlockExplorer: GetCurrentBlockExplorer,
) : ViewModel() {

    private val walletConnect = WalletConnect()
    private val state = MutableStateFlow(RequestViewModelState())
    private var requestJob: Job? = null
    val sceneState = state.map { it.toSceneState() }.stateIn(viewModelScope, SharingStarted.Eagerly, RequestSceneState.Loading)

    fun onRequest(
        sessionRequest: Wallet.Model.SessionRequest,
        verifyContext: Wallet.Model.VerifyContext,
        onNotify: (BridgeRequestError) -> Unit
    ) {
        requestJob?.cancel()
        state.update { it.copy(sessionRequest = sessionRequest) }
        val job = viewModelScope.launch {
            try {
                val connection = bridgeRepository.getConnectionByTopic(sessionRequest.topic)
                if (connection == null) {
                    rejectRequest(sessionRequest)
                    return@launch
                }

                val appMetadata = connection.session.metadata
                validateSession(appMetadata.url, verifyContext)
                val chainId = sessionRequest.chainId ?: throw BridgeRequestError.UnresolvedChainId
                val sessionDomain = appMetadata.url
                val action = walletConnect.parseRequest(
                    topic = sessionRequest.topic,
                    method = sessionRequest.request.method,
                    params = sessionRequest.request.params,
                    chainId = chainId,
                    domain = sessionDomain,
                )
                if (action is WalletConnectAction.ChainOperation) {
                    when (action.operation) {
                        uniffi.gemstone.WalletConnectChainOperation.AddChain,
                        is uniffi.gemstone.WalletConnectChainOperation.SwitchChain -> {
                            respondWithNull(sessionRequest)
                        }

                        uniffi.gemstone.WalletConnectChainOperation.GetChainId -> {
                            respondError(
                                topic = sessionRequest.topic,
                                id = sessionRequest.request.id,
                                code = -32601,
                                message = "The method does not exist / is not available."
                            )
                        }
                    }
                    return@launch
                }

                val wallet = walletsRepository.getWallet(connection.wallet.id).firstOrNull()
                if (wallet == null) {
                    rejectRequest(sessionRequest)
                    return@launch
                }
                val chain = Chain.getNamespace(chainId)
                    ?: throw BridgeRequestError.ChainUnsupported

                validateChain(chain, connection.session)

                val account: Account = wallet.getAccount(chain) ?: throw BridgeRequestError.ChainUnsupported

                currentCoroutineContext().ensureActive()
                val request = when (action) {
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

                    is WalletConnectAction.ChainOperation -> error("Immediate WalletConnect responses must be handled before request resolution")
                    is WalletConnectAction.Unsupported -> throw BridgeRequestError.MethodUnsupported
                }
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
                    else -> state.update { it.copy(error = err.message ?: "Request failed") }
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

    private fun respondWithNull(request: Wallet.Model.SessionRequest) {
        response(request.topic, request.request.id, "null")
    }

    fun onTransactionResult(result: String) {
        viewModelScope.launch(Dispatchers.IO) {
            val request = state.value.request as? WCRequest.Transaction ?: return@launch
            val response = try {
                request.execute(result)
            } catch (err: Throwable) {
                state.update { it.copy(error = err.message ?: "Request failed") }
                return@launch
            }
            response(request.topic, request.requestId, response)
        }
    }

    fun onSign() {
        val request = (state.value.request as? WCRequest.SignMessage) ?: return
        val wallet = state.value.wallet ?: return
        val chain = state.value.chain ?: return

        viewModelScope.launch(Dispatchers.IO) {
            val password = passwordStore.getPassword(wallet.id)
            val privateKey = loadPrivateKeyOperator(wallet, chain, password)
            val sign = try {
                request.execute(privateKey)
            } catch (err: Throwable) {
                state.update { it.copy(error = err.message ?: "Sign failed") }
                return@launch
            } finally {
                Arrays.fill(privateKey, 0)
            }
            response(request.sessionRequest.topic, request.sessionRequest.request.id, sign)
        }
    }

    private fun validateSession(
        metadataUrl: String,
        verifyContext: Wallet.Model.VerifyContext
    ) {
        val validation = walletConnect.validateOrigin(
            metadataUrl,
            verifyContext.origin,
            verifyContext.map()
        )
        when (validation) {
            WalletConnectionVerificationStatus.UNKNOWN,
            WalletConnectionVerificationStatus.VERIFIED -> {
                return
            }
            WalletConnectionVerificationStatus.INVALID,
            WalletConnectionVerificationStatus.MALICIOUS -> {
                throw BridgeRequestError.ScamSession
            }
        }
    }

    private fun response(topic: String, id: Long, payload: String) {
        WalletKit.respondSessionRequest(
            params = Wallet.Params.SessionRequestResponse(
                sessionTopic = topic,
                jsonRpcResponse = Wallet.Model.JsonRpcResponse.JsonRpcResult(id, payload)
            ),
            onSuccess = { state.update { it.copy(canceled = true) } },
            onError = { error ->
                state.update { it.copy(error = error.throwable.message ?: "Request failed") }
            }
        )
    }

    private fun respondError(topic: String, id: Long, code: Int, message: String) {
        WalletKit.respondSessionRequest(
            params = Wallet.Params.SessionRequestResponse(
                sessionTopic = topic,
                jsonRpcResponse = Wallet.Model.JsonRpcResponse.JsonRpcError(id, code, message)
            ),
            onSuccess = { state.update { it.copy(canceled = true) } },
            onError = { error ->
                state.update { it.copy(error = error.throwable.message ?: "Request failed") }
            }
        )
    }

    fun onReject() {
        requestJob?.cancel()
        val sessionRequest = state.value.sessionRequest ?: return
        rejectRequest(sessionRequest)
    }

    private fun handleRequestFailure(
        sessionRequest: Wallet.Model.SessionRequest,
        error: BridgeRequestError,
        onNotify: (BridgeRequestError) -> Unit
    ) {
        if (error is BridgeRequestError.ScamSession) {
            onNotify(error)
        }
        rejectRequest(sessionRequest)
    }

    private fun rejectRequest(sessionRequest: Wallet.Model.SessionRequest) {
        val result = Wallet.Params.SessionRequestResponse(
            sessionTopic = sessionRequest.topic,
            jsonRpcResponse = Wallet.Model.JsonRpcResponse.JsonRpcError(
                id = sessionRequest.request.id,
                code = 4001,
                message = "User rejected the request"
            )
        )

        WalletKit.respondSessionRequest(
            result,
            onSuccess = { state.update { it.copy(canceled = true) } },
            onError = { error ->
                state.update { it.copy(error = error.throwable.message ?: "Request failed") }
            }
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
}

private data class RequestViewModelState(
    val sessionRequest: Wallet.Model.SessionRequest? = null,
    val error: String? = null,
    val canceled: Boolean = false,
    val wallet: com.wallet.core.primitives.Wallet? = null,
    val request: WCRequest? = null,
    val chain: Chain? = null,
) {
    fun toSceneState(): RequestSceneState {
        if (canceled) {
            return RequestSceneState.Cancel
        }
        if (error != null) {
            return RequestSceneState.Error(error)
        }
        if (request == null) {
            return RequestSceneState.Loading
        }
        wallet ?: return RequestSceneState.Loading

        return RequestSceneState.Request(walletName = wallet.name, request = request)
    }
}

sealed interface RequestSceneState {

    data object Loading : RequestSceneState

    data object Cancel : RequestSceneState

    class Error(val message: String) : RequestSceneState

    class Request(
        val walletName: String,
        val request: WCRequest,
    ) : RequestSceneState
}
