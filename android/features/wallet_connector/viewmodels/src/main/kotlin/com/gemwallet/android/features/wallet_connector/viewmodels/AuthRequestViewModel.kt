package com.gemwallet.android.features.wallet_connector.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectAuthPayloadParams
import com.gemwallet.android.application.wallet_connect.WalletConnectAuthenticationRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnectAuthentication
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.wallet_connector.viewmodels.models.ReviewTexts
import com.gemwallet.android.features.wallet_connector.viewmodels.models.WalletConnectReviewModel
import com.gemwallet.android.features.wallet_connector.viewmodels.models.map
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.WalletSectionUIModel
import com.gemwallet.android.ui.components.list_item.uiModel
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemConnectionRow
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemSignMessageServiceInterface
import uniffi.gemstone.GemSimulationPayloadRow
import uniffi.gemstone.GemWalletConnectAuthAccount
import uniffi.gemstone.GemWalletConnectException
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.SignDigestType
import uniffi.gemstone.SignMessage
import uniffi.gemstone.applicationConnectionRow
import uniffi.gemstone.walletSections
import javax.inject.Inject

@HiltViewModel
class AuthRequestViewModel @Inject constructor(
    private val approveWalletConnectAuthentication: ApproveWalletConnectAuthentication,
    private val activeRequest: ActiveWalletConnectRequest,
    private val walletConnectService: GemWalletConnectServiceInterface,
    private val signMessageService: GemSignMessageServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private var authRequest: WalletConnectAuthenticationRequest? = null
    private var hasResponded = false

    private val _state = MutableStateFlow<AuthRequestUIState>(AuthRequestUIState.Loading)
    val state: StateFlow<AuthRequestUIState> = _state.asStateFlow()

    val buttonState: StateFlow<ButtonState> = state
        .map { buttonState(loading = it is AuthRequestUIState.Approving) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Enabled)

    fun onRequest(request: WalletConnectAuthenticationRequest, verifyContext: WalletConnectVerifyContext, onNotify: (String) -> Unit) {
        if (authRequest?.id == request.id) {
            return
        }
        authRequest = request
        hasResponded = false
        _state.update { AuthRequestUIState.Loading }
        viewModelScope.launch {
            val content = withContext(ioDispatcher) {
                runCatchingCancellable { requestContent(request, verifyContext) }
            }.getOrElse { err ->
                if (!isActiveRequest(request)) {
                    return@launch
                }
                if (err is GemWalletConnectException.InvalidOrigin) {
                    onNotify(err.errorText().text(context))
                    hasResponded = true
                    approveWalletConnectAuthentication.rejectAuthentication(request)
                    finish(request)
                } else {
                    rejectRequest(request, AuthRequestUIState.Error(err.errorText().text(context)))
                }
                return@launch
            }
            if (isActiveRequest(request)) {
                _state.update { content }
            }
        }
    }

    fun onWalletSelected(walletId: WalletId) {
        val current = _state.value as? AuthRequestUIState.Request ?: return
        val wallet = current.availableWallets.firstOrNull { it.id == walletId } ?: return
        val request = authRequest ?: return
        val approval = runCatching {
            buildApproval(request, wallet)
        }.getOrElse { err ->
            _state.update { AuthRequestUIState.Error(err.errorText().text(context)) }
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
        val current = _state.value as? AuthRequestUIState.Request ?: return
        if (hasResponded) {
            return
        }
        val approval = current.approval
        _state.update { AuthRequestUIState.Approving(current) }

        viewModelScope.launch(ioDispatcher) {
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
                    onError = { error ->
                        if (authRequest?.id == request.id) {
                            _state.update { AuthRequestUIState.Error(error.text(context)) }
                        }
                    },
                )
            } catch (err: Throwable) {
                if (authRequest?.id == request.id) {
                    _state.update { AuthRequestUIState.Error(err.errorText().text(context)) }
                }
            }
        }
    }

    fun onReject() {
        if (_state.value is AuthRequestUIState.Approving) {
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

    private fun rejectRequest(request: WalletConnectAuthenticationRequest, errorState: AuthRequestUIState.Error) {
        if (!isActiveRequest(request)) {
            return
        }
        hasResponded = true
        approveWalletConnectAuthentication.rejectAuthentication(request)
        _state.update { errorState }
    }

    private fun isActiveRequest(request: WalletConnectAuthenticationRequest): Boolean = authRequest?.id == request.id && !hasResponded

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
        _state.update { AuthRequestUIState.Loading }
    }

    private suspend fun requestContent(request: WalletConnectAuthenticationRequest, verifyContext: WalletConnectVerifyContext): AuthRequestUIState.Request {
        val prepared = walletConnectService.prepareSessionProposal(
            requiredChainIds = emptyList(),
            optionalChainIds = walletConnectService.authenticationChainIds(request.payloadParams.chains),
            metadata = walletConnectService.applicationMetadata(
                request.metadata?.name.orEmpty(),
                request.metadata?.description.orEmpty(),
                request.metadata?.url.orEmpty(),
                listOfNotNull(request.metadata?.icon),
            ),
            origin = verifyContext.origin,
            validation = verifyContext.map(),
        )
        val selectedWallet = prepared.proposal.defaultWallet.toPrimitives()
        return AuthRequestUIState.Request(
            texts = ReviewTexts(context),
            peer = applicationConnectionRow(prepared.proposal.metadata),
            availableWallets = prepared.proposal.wallets.map { it.toPrimitives() },
            availableWalletSections = walletSections(prepared.proposal.wallets).map { it.uiModel(context) },
            selectedWallet = selectedWallet,
            approval = buildApproval(request, selectedWallet),
        )
    }

    private fun buildApproval(request: WalletConnectAuthenticationRequest, wallet: Wallet): AuthApproval {
        val supportedAccounts = supportedAccounts(wallet, request)
        val selectedAccount = supportedAccounts.firstOrNull()
            ?: throw IllegalStateException("Requested chains are not supported")
        val supportedChains = supportedAccounts.map { it.chainId }
        val payloadParams = approveWalletConnectAuthentication.authPayloadParams(
            payloadParams = request.payloadParams,
            supportedChains = supportedChains,
            supportedMethods = walletConnectService.authenticationMethods(),
        )
        val issuer = selectedAccount.issuer
        val message = approveWalletConnectAuthentication.authMessage(payloadParams, issuer)
        val payloadPreview = payloadPreview(selectedAccount.account.toPrimitives().chain, message)

        return AuthApproval(
            wallet = wallet,
            account = selectedAccount.account.toPrimitives(),
            payloadParams = payloadParams,
            issuer = issuer,
            message = message,
            title = payloadPreview.title,
            primaryPayloadFields = payloadPreview.primaryFields,
            secondaryPayloadFields = payloadPreview.secondaryFields,
        )
    }

    private fun supportedAccounts(wallet: Wallet, request: WalletConnectAuthenticationRequest): List<GemWalletConnectAuthAccount> = walletConnectService.authenticationAccounts(request.payloadParams.chains, wallet.toGem())

    private fun payloadPreview(chain: Chain, message: String): AuthPayloadPreview = signMessageService.payloadPreview(
        SignMessage(
            chain = chain.string,
            signType = SignDigestType.SIWE,
            data = message.toByteArray(),
        ),
    )?.let { preview ->
        AuthPayloadPreview(
            title = preview.title,
            primaryFields = preview.primary,
            secondaryFields = preview.secondary,
        )
    } ?: AuthPayloadPreview()

    private suspend fun signAuthMessage(wallet: Wallet, chain: Chain, message: String): String = walletConnectService.signMessage(
        wallet.id.id,
        SignMessage(
            chain = chain.string,
            signType = SignDigestType.SIWE,
            data = message.toByteArray(),
        ),
    )
}

sealed interface AuthRequestUIState {

    data object Loading : AuthRequestUIState

    class Error(val text: String) : AuthRequestUIState

    sealed interface Content :
        AuthRequestUIState,
        WalletConnectReviewModel {
        val peer: GemConnectionRow
        val availableWallets: List<Wallet>
        val availableWalletSections: List<WalletSectionUIModel>
        val selectedWallet: Wallet
        val approval: AuthApproval
        val texts: ReviewTexts

        val appListItem: ListItemModel get() = ListItemModel(title = texts.app, subtitle = peer.title)
        val walletListItem: ListItemModel get() = ListItemModel(title = texts.wallet, subtitle = selectedWallet.name)
        override val viewFullMessageListItem: ListItemModel get() = ListItemModel(title = texts.viewFullMessage)

        override val icon: String? get() = peer.iconUrl
        override val name: String get() = peer.title
        override val uri: String get() = peer.host.orEmpty()
        override val chain: Chain get() = approval.chain
        override val primaryPayloadFields: List<GemSimulationPayloadRow> get() = approval.primaryPayloadFields
        override val secondaryPayloadFields: List<GemSimulationPayloadRow> get() = approval.secondaryPayloadFields
        override val title: GemLocalizedText get() = approval.title
        override val message: String get() = approval.message
    }

    data class Request(
        override val peer: GemConnectionRow,
        override val availableWallets: List<Wallet>,
        override val availableWalletSections: List<WalletSectionUIModel>,
        override val selectedWallet: Wallet,
        override val approval: AuthApproval,
        override val texts: ReviewTexts,
    ) : Content

    data class Approving(private val request: Request) : Content {
        override val peer: GemConnectionRow get() = request.peer
        override val availableWallets: List<Wallet> get() = request.availableWallets
        override val availableWalletSections: List<WalletSectionUIModel> get() = request.availableWalletSections
        override val selectedWallet: Wallet get() = request.selectedWallet
        override val approval: AuthApproval get() = request.approval
        override val texts: ReviewTexts get() = request.texts
    }
}

data class AuthApproval(
    val wallet: Wallet,
    val account: Account,
    val payloadParams: WalletConnectAuthPayloadParams,
    val issuer: String,
    val message: String,
    val title: GemLocalizedText,
    val primaryPayloadFields: List<GemSimulationPayloadRow>,
    val secondaryPayloadFields: List<GemSimulationPayloadRow>,
) {
    val chain: Chain get() = account.chain
}

private data class AuthPayloadPreview(val title: GemLocalizedText = GemLocalizedText.ReviewRequest, val primaryFields: List<GemSimulationPayloadRow> = emptyList(), val secondaryFields: List<GemSimulationPayloadRow> = emptyList())
