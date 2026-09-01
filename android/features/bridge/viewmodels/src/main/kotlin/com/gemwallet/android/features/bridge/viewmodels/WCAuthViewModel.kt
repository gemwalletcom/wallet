package com.gemwallet.android.features.bridge.viewmodels

import uniffi.gemstone.GemApplicationMetadataService
import uniffi.gemstone.GemChainService
import uniffi.gemstone.GemWalletConnectService
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.getKeystorePassword
import com.gemwallet.android.application.wallet_connect.cases.PrepareSessionProposal
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.blockchain.services.GemSignMessageOperator
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnectAuthentication
import com.gemwallet.android.application.wallet_connect.ChainNamespace
import com.gemwallet.android.application.wallet_connect.WalletConnectAuthPayloadParams
import com.gemwallet.android.application.wallet_connect.WalletConnectAuthenticationRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import com.gemwallet.android.application.wallet_connect.fromWalletConnectChainId
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.features.bridge.viewmodels.model.map
import com.gemwallet.android.features.bridge.viewmodels.model.BridgeRequestError
import com.gemwallet.android.features.bridge.viewmodels.model.SessionUI
import com.gemwallet.android.features.bridge.viewmodels.model.WalletConnectOriginVerifier
import com.gemwallet.android.features.bridge.viewmodels.model.WalletConnectReviewModel
import com.gemwallet.android.features.bridge.viewmodels.model.toSessionUI
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.models.buttonState
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.MessageSigner
import uniffi.gemstone.SignDigestType
import uniffi.gemstone.SignMessage
import javax.inject.Inject

@HiltViewModel
class WCAuthViewModel @Inject constructor(
    private val applicationMetadataService: GemApplicationMetadataService,
    private val approveWalletConnectAuthentication: ApproveWalletConnectAuthentication,
    private val prepareSessionProposal: PrepareSessionProposal,
    private val passwordStore: PasswordStore,
    private val signMessageOperator: GemSignMessageOperator,
    private val originVerifier: WalletConnectOriginVerifier,
    private val activeRequest: ActiveWalletConnectRequest,
    private val walletConnectService: GemWalletConnectService,
    private val chainService: GemChainService,
) : ViewModel() {

    private var authRequest: WalletConnectAuthenticationRequest? = null
    private var hasResponded = false

    private val _state = MutableStateFlow<AuthSceneState>(AuthSceneState.Loading)
    val state: StateFlow<AuthSceneState> = _state.asStateFlow()

    val buttonState: StateFlow<ButtonState> = state
        .map { buttonState(loading = it is AuthSceneState.Approving) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Enabled)

    fun onRequest(
        request: WalletConnectAuthenticationRequest,
        verifyContext: WalletConnectVerifyContext,
        onNotify: (BridgeRequestError) -> Unit,
    ) {
        authRequest = request
        hasResponded = false
        _state.update { AuthSceneState.Loading }
        if (originVerifier.isRejected(request.metadata?.url, verifyContext)) {
            onNotify(BridgeRequestError.MaliciousSession)
            hasResponded = true
            approveWalletConnectAuthentication.rejectAuthentication(request)
            finish(request)
            return
        }
        viewModelScope.launch(Dispatchers.IO) {
            try {
                val prepared = prepareSessionProposal(
                    name = request.metadata?.name.orEmpty(),
                    description = request.metadata?.description.orEmpty(),
                    url = request.metadata?.url.orEmpty(),
                    icons = listOfNotNull(request.metadata?.icon),
                    requiredChainIds = emptyList(),
                    optionalChainIds = request.ethereumChainIds(),
                    origin = verifyContext.origin,
                    validation = verifyContext.map(),
                )

                if (!isActiveRequest(request)) {
                    return@launch
                }
                val selectedWallet = prepared.proposal.defaultWallet
                val approval = buildApproval(request, selectedWallet)

                if (!isActiveRequest(request)) {
                    return@launch
                }
                _state.update {
                    AuthSceneState.Request(
                        peer = prepared.proposal.metadata.toSessionUI(applicationMetadataService),
                        availableWallets = prepared.proposal.wallets,
                        selectedWallet = selectedWallet,
                        approval = approval,
                    )
                }
            } catch (err: Throwable) {
                if (isActiveRequest(request)) {
                    rejectRequest(request, AuthSceneState.Error(err.message, err))
                }
            }
        }
    }

    fun onWalletSelected(walletId: WalletId) {
        val current = _state.value as? AuthSceneState.Request ?: return
        val wallet = current.availableWallets.firstOrNull { it.id == walletId } ?: return
        val request = authRequest ?: return
        val approval = runCatching {
            buildApproval(request, wallet)
        }.getOrElse { err ->
            _state.update { AuthSceneState.Error(err.message) }
            return
        }

        _state.update {
            current.copy(
                selectedWallet = wallet,
                approval = approval,
            )
        }
    }

    fun onApprove() {
        val request = authRequest ?: return
        val current = _state.value as? AuthSceneState.Request ?: return
        if (hasResponded) {
            return
        }
        val approval = current.approval
        _state.update { AuthSceneState.Approving(current) }

        viewModelScope.launch(Dispatchers.IO) {
            try {
                if (!isActiveRequest(request)) {
                    return@launch
                }
                val signature = signAuthMessage(
                    wallet = approval.wallet,
                    chain = approval.account.chain,
                    message = approval.message,
                )
                if (!isActiveRequest(request)) {
                    return@launch
                }
                val authObject = approveWalletConnectAuthentication.authObject(
                    payloadParams = approval.payloadParams,
                    issuer = approval.issuer,
                    signature = signature,
                )
                approveWalletConnectAuthentication.approveAuthentication(
                    request = request,
                    auths = listOf(authObject),
                    wallet = approval.wallet,
                    onSuccess = {
                        if (authRequest?.id == request.id) {
                            hasResponded = true
                            finish(request)
                        }
                    },
                    onError = { message ->
                        if (authRequest?.id == request.id) {
                            _state.update { AuthSceneState.Error(message) }
                        }
                    },
                )
            } catch (err: Throwable) {
                if (authRequest?.id == request.id) {
                    _state.update { AuthSceneState.Error(err.message) }
                }
            }
        }
    }

    fun onReject() {
        if (_state.value is AuthSceneState.Approving) {
            return
        }
        val request = authRequest
        if (request == null || hasResponded) {
            finish()
            return
        }
        hasResponded = true
        approveWalletConnectAuthentication.rejectAuthentication(request)
        finish()
    }

    private fun rejectRequest(
        request: WalletConnectAuthenticationRequest,
        errorState: AuthSceneState.Error,
    ) {
        if (!isActiveRequest(request)) {
            return
        }
        hasResponded = true
        approveWalletConnectAuthentication.rejectAuthentication(request)
        _state.update { errorState }
    }

    private fun isActiveRequest(request: WalletConnectAuthenticationRequest): Boolean {
        return authRequest?.id == request.id && !hasResponded
    }

    private fun finish(request: WalletConnectAuthenticationRequest) {
        if (activeRequest.finish(request)) {
            reset()
        }
    }

    private fun finish() {
        reset()
        activeRequest.finish()
    }

    private fun reset() {
        authRequest = null
        _state.update { AuthSceneState.Loading }
    }

    private fun buildApproval(
        request: WalletConnectAuthenticationRequest,
        wallet: Wallet,
    ): AuthApproval {
        val supportedAccounts = supportedAccounts(wallet, request)
        val selectedAccount = supportedAccounts.firstOrNull()
            ?: throw IllegalStateException("Requested chains are not supported")
        val supportedChains = supportedAccounts.map { it.chainId }.distinct()
        val payloadParams = approveWalletConnectAuthentication.authPayloadParams(
            payloadParams = request.payloadParams,
            supportedChains = supportedChains,
            supportedMethods = ChainNamespace.Eip155.methodIds,
        )
        val issuer = selectedAccount.issuer
        val message = approveWalletConnectAuthentication.authMessage(payloadParams, issuer)
        val payloadPreview = payloadPreview(selectedAccount.account.chain, message)

        return AuthApproval(
            wallet = wallet,
            account = selectedAccount.account,
            payloadParams = payloadParams,
            issuer = issuer,
            message = message,
            primaryPayloadFields = payloadPreview.primaryFields,
            secondaryPayloadFields = payloadPreview.secondaryFields,
        )
    }

    private fun supportedAccounts(
        wallet: Wallet,
        request: WalletConnectAuthenticationRequest,
    ): List<AuthAccount> {
        return request.ethereumChainIds().mapNotNull { chainId ->
            val chain = Chain.fromWalletConnectChainId(chainService, chainId) ?: return@mapNotNull null
            val account = wallet.getAccount(chain) ?: return@mapNotNull null
            AuthAccount(account = account, chainId = chainId)
        }
    }

    private fun WalletConnectAuthenticationRequest.ethereumChainIds(): List<String> {
        return walletConnectService.authenticationChainIds(payloadParams.chains)
    }

    private fun payloadPreview(
        chain: Chain,
        message: String,
    ): AuthPayloadPreview {
        val signer = MessageSigner(
            SignMessage(
                chain = chain.string,
                signType = SignDigestType.SIWE,
                data = message.toByteArray(),
            )
        )
        return try {
            signer.payloadPreview(emptyList())?.let { preview ->
                AuthPayloadPreview(
                    primaryFields = preview.primary.map { PayloadField(field = it.decodeJson(), chain = chain) },
                    secondaryFields = preview.secondary.map { PayloadField(field = it.decodeJson(), chain = chain) },
                )
            } ?: AuthPayloadPreview()
        } catch (_: Throwable) {
            AuthPayloadPreview()
        } finally {
            signer.close()
        }
    }

    private suspend fun signAuthMessage(
        wallet: Wallet,
        chain: Chain,
        message: String,
    ): String {
        val signer = MessageSigner(
            SignMessage(
                chain = chain.string,
                signType = SignDigestType.SIWE,
                data = message.toByteArray(),
            )
        )
        return try {
            signMessageOperator.sign(signer, wallet, passwordStore.getKeystorePassword())
        } finally {
            signer.close()
        }
    }

}

sealed interface AuthSceneState {

    data object Loading : AuthSceneState

    class Error(val message: String?, val cause: Throwable? = null) : AuthSceneState

    sealed interface Content : AuthSceneState, WalletConnectReviewModel {
        val peer: SessionUI
        val availableWallets: List<Wallet>
        val selectedWallet: Wallet
        val approval: AuthApproval

        override val icon: String get() = peer.icon
        override val name: String get() = peer.name
        override val uri: String get() = peer.uri
        override val chain: Chain get() = approval.chain
        override val primaryPayloadFields: List<PayloadField> get() = approval.primaryPayloadFields
        override val secondaryPayloadFields: List<PayloadField> get() = approval.secondaryPayloadFields
        override val message: String get() = approval.message
    }

    data class Request(
        override val peer: SessionUI,
        override val availableWallets: List<Wallet>,
        override val selectedWallet: Wallet,
        override val approval: AuthApproval,
    ) : Content

    data class Approving(
        private val request: Request,
    ) : Content {
        override val peer: SessionUI get() = request.peer
        override val availableWallets: List<Wallet> get() = request.availableWallets
        override val selectedWallet: Wallet get() = request.selectedWallet
        override val approval: AuthApproval get() = request.approval
    }
}

data class AuthApproval(
    val wallet: Wallet,
    val account: Account,
    val payloadParams: WalletConnectAuthPayloadParams,
    val issuer: String,
    val message: String,
    val primaryPayloadFields: List<PayloadField>,
    val secondaryPayloadFields: List<PayloadField>,
) {
    val chain: Chain get() = account.chain
}

private data class AuthAccount(
    val account: Account,
    val chainId: String,
) {
    val issuer: String get() = "did:pkh:$chainId:${account.address}"
}

private data class AuthPayloadPreview(
    val primaryFields: List<PayloadField> = emptyList(),
    val secondaryFields: List<PayloadField> = emptyList(),
)
