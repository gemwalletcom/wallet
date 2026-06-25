package com.gemwallet.android.features.bridge.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.blockchain.gemstone.toPrimitives
import com.gemwallet.android.blockchain.services.GemSignMessageOperator
import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.data.repositories.bridge.ChainNamespace
import com.gemwallet.android.data.repositories.bridge.WalletConnectAuthPayloadParams
import com.gemwallet.android.data.repositories.bridge.WalletConnectAuthenticationRequest
import com.gemwallet.android.data.repositories.bridge.WalletConnectVerifyContext
import com.gemwallet.android.data.repositories.bridge.fromWalletConnectChainId
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toChainType
import com.gemwallet.android.ext.walletConnectAppName
import com.gemwallet.android.features.bridge.viewmodels.model.SessionUI
import com.gemwallet.android.features.bridge.viewmodels.model.WalletConnectOriginVerifier
import com.gemwallet.android.features.bridge.viewmodels.model.toSessionUI
import com.gemwallet.android.ui.models.PayloadField
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainType
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletConnectionSessionAppMetadata
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.MessageSigner
import uniffi.gemstone.SignDigestType
import uniffi.gemstone.SignMessage
import javax.inject.Inject

@HiltViewModel
class WCAuthViewModel @Inject constructor(
    private val sessionRepository: SessionRepository,
    private val bridgesRepository: BridgesRepository,
    private val walletsRepository: WalletsRepository,
    private val passwordStore: PasswordStore,
    private val signMessageOperator: GemSignMessageOperator,
    private val originVerifier: WalletConnectOriginVerifier,
) : ViewModel() {

    private var authRequest: WalletConnectAuthenticationRequest? = null
    private var hasResponded = false

    private val _state = MutableStateFlow<AuthSceneState>(AuthSceneState.Loading)
    val state: StateFlow<AuthSceneState> = _state.asStateFlow()

    fun onRequest(
        request: WalletConnectAuthenticationRequest,
        verifyContext: WalletConnectVerifyContext,
    ) {
        authRequest = request
        hasResponded = false
        _state.update { AuthSceneState.Loading }
        viewModelScope.launch(Dispatchers.IO) {
            try {
                val verification = originVerifier.verify(request.metadata?.url, verifyContext)
                if (!isActiveRequest(request)) {
                    return@launch
                }
                if (verification.isScam) {
                    rejectRequest(request, AuthSceneState.ScamCanceled)
                    return@launch
                }

                val availableWallets = (walletsRepository.getAll().firstOrNull() ?: emptyList())
                    .filter { wallet ->
                        wallet.type != WalletType.View && supportedAccounts(wallet, request).isNotEmpty()
                    }
                    .sortedBy { it.type }

                if (!isActiveRequest(request)) {
                    return@launch
                }
                if (availableWallets.isEmpty()) {
                    rejectRequest(request, AuthSceneState.Error("Requested chains are not supported"))
                    return@launch
                }

                val currentWallet = sessionRepository.session().firstOrNull()?.wallet
                val selectedWallet = availableWallets.firstOrNull { currentWallet?.id == it.id } ?: availableWallets.first()
                val approval = buildApproval(request, selectedWallet)

                if (!isActiveRequest(request)) {
                    return@launch
                }
                _state.update {
                    AuthSceneState.Request(
                        peer = request.toSessionUI(),
                        availableWallets = availableWallets,
                        selectedWallet = selectedWallet,
                        approval = approval,
                    )
                }
            } catch (err: Throwable) {
                if (isActiveRequest(request)) {
                    rejectRequest(request, AuthSceneState.Error(err.message ?: "Authentication failed"))
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
            _state.update { AuthSceneState.Error(err.message ?: "Authentication failed") }
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
                val authObject = bridgesRepository.generateAuthObject(
                    payloadParams = approval.payloadParams,
                    issuer = approval.issuer,
                    signature = signature,
                )
                bridgesRepository.approveAuthentication(
                    request = request,
                    auths = listOf(authObject),
                    wallet = approval.wallet,
                    onSuccess = {
                        if (authRequest?.id == request.id) {
                            hasResponded = true
                            _state.update { AuthSceneState.Canceled }
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
                    _state.update { AuthSceneState.Error(err.message ?: "Authentication failed") }
                }
            }
        }
    }

    fun onReject() {
        if (_state.value is AuthSceneState.Approving) {
            return
        }
        val request = authRequest
        if (request == null) {
            _state.update { AuthSceneState.Canceled }
            return
        }
        if (hasResponded) {
            _state.update { AuthSceneState.Canceled }
            return
        }
        hasResponded = true
        bridgesRepository.rejectAuthentication(request)
        _state.update { AuthSceneState.Canceled }
    }

    private fun rejectRequest(
        request: WalletConnectAuthenticationRequest,
        state: AuthSceneState,
    ) {
        if (!isActiveRequest(request)) {
            return
        }
        hasResponded = true
        bridgesRepository.rejectAuthentication(request)
        _state.update { state }
    }

    private fun isActiveRequest(request: WalletConnectAuthenticationRequest): Boolean {
        return authRequest?.id == request.id && !hasResponded
    }

    private fun buildApproval(
        request: WalletConnectAuthenticationRequest,
        wallet: Wallet,
    ): AuthApproval {
        val supportedAccounts = supportedAccounts(wallet, request)
        val selectedAccount = supportedAccounts.firstOrNull()
            ?: throw IllegalStateException("Requested chains are not supported")
        val supportedChains = supportedAccounts.map { it.chainId }.distinct()
        val payloadParams = bridgesRepository.generateAuthPayloadParams(
            payloadParams = request.payloadParams,
            supportedChains = supportedChains,
            supportedMethods = ChainNamespace.Eip155.methodIds,
        )
        val issuer = selectedAccount.issuer
        val message = bridgesRepository.formatAuthMessage(payloadParams, issuer)
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
        val requestedChains = request.payloadParams.chains.toSet()
        if (requestedChains.isEmpty()) {
            return emptyList()
        }

        return requestedChains.mapNotNull { chainId ->
            val chain = Chain.fromWalletConnectChainId(chainId) ?: return@mapNotNull null
            if (chain.toChainType() != ChainType.Ethereum) {
                return@mapNotNull null
            }
            val account = wallet.getAccount(chain) ?: return@mapNotNull null
            AuthAccount(account = account, chainId = chainId)
        }
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
                    primaryFields = preview.primary.map { PayloadField(it.toPrimitives()) },
                    secondaryFields = preview.secondary.map { PayloadField(it.toPrimitives()) },
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
            signMessageOperator.sign(signer, wallet, passwordStore.getPassword(wallet.id.id))
        } finally {
            signer.close()
        }
    }

    private fun WalletConnectAuthenticationRequest.toSessionUI(): SessionUI {
        return WalletConnectionSessionAppMetadata(
            name = walletConnectAppName(metadata?.name, metadata?.url),
            description = metadata?.description ?: "",
            url = metadata?.url ?: "",
            icon = metadata?.icon ?: "",
        ).toSessionUI()
    }

}

sealed interface AuthSceneState {

    data object Loading : AuthSceneState

    data object Canceled : AuthSceneState

    data object ScamCanceled : AuthSceneState

    class Error(val message: String) : AuthSceneState

    sealed interface Content : AuthSceneState {
        val peer: SessionUI
        val availableWallets: List<Wallet>
        val selectedWallet: Wallet
        val approval: AuthApproval
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
    val hasPayload: Boolean get() = primaryPayloadFields.isNotEmpty() || secondaryPayloadFields.isNotEmpty()
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
