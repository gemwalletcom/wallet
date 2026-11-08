package com.gemwallet.android.features.wallet_connector.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnectAuthentication
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.listItem
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.wallet_connector.viewmodels.models.ReviewTexts
import com.gemwallet.android.features.wallet_connector.viewmodels.models.WalletConnectReviewModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
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
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemSignMessageServiceInterface
import uniffi.gemstone.GemWalletConnectAuthentication
import uniffi.gemstone.GemWalletConnectException
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.GemWalletSection
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

    private val phase = MutableStateFlow<AuthRequestPhase>(AuthRequestPhase.Idle)

    private val _state = MutableStateFlow<AuthRequestUIState>(AuthRequestUIState.Loading)
    val state: StateFlow<AuthRequestUIState> = _state.asStateFlow()

    val buttonState: StateFlow<ButtonState> = state
        .map { buttonState(loading = it is AuthRequestUIState.Approving) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Enabled)

    fun onRequest(request: GemWalletConnectAuthentication, onError: (String) -> Unit) {
        if (phase.value.request?.id == request.id) {
            return
        }
        rejectOnce()
        phase.value = AuthRequestPhase.Waiting(request)
        _state.update { AuthRequestUIState.Loading }
        viewModelScope.launch {
            val content = withContext(ioDispatcher) {
                runCatchingCancellable { requestContent(request) }
            }.getOrElse { err ->
                if (!answer(request)) {
                    return@launch
                }
                approveWalletConnectAuthentication.rejectAuthentication(request)
                if (err is GemWalletConnectException.InvalidOrigin) {
                    onError(err.errorText().text(context))
                    finish(request)
                } else {
                    _state.update { AuthRequestUIState.Error(err.errorText().text(context)) }
                }
                return@launch
            }
            if (isWaiting(request)) {
                _state.update { content }
            }
        }
    }

    fun onWalletSelected(walletId: WalletId) {
        val current = _state.value as? AuthRequestUIState.Request ?: return
        val wallet = current.availableWallets.firstOrNull { it.id == walletId } ?: return
        val request = phase.value.request ?: return
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
        val request = (phase.value as? AuthRequestPhase.Waiting)?.request ?: return
        val current = _state.value as? AuthRequestUIState.Request ?: return
        if (!phase.compareAndSet(AuthRequestPhase.Waiting(request), AuthRequestPhase.Signing(request))) {
            return
        }
        val approval = current.approval
        _state.update { AuthRequestUIState.Approving(current) }

        viewModelScope.launch(ioDispatcher) {
            val signature = runCatchingCancellable { signMessageService.sign(approval.wallet.id.id, approval.signMessage) }.getOrElse { err ->
                if (phase.compareAndSet(AuthRequestPhase.Signing(request), AuthRequestPhase.Waiting(request))) {
                    _state.update { AuthRequestUIState.Error(err.errorText().text(context)) }
                }
                return@launch
            }
            if (!phase.compareAndSet(AuthRequestPhase.Signing(request), AuthRequestPhase.Sending(request))) {
                return@launch
            }
            approveWalletConnectAuthentication.approveAuthentication(
                authentication = request,
                wallet = approval.wallet,
                signature = signature,
                onSuccess = {
                    if (phase.compareAndSet(AuthRequestPhase.Sending(request), AuthRequestPhase.Answered(request))) {
                        finish(request)
                    }
                },
                onError = { error ->
                    if (phase.compareAndSet(AuthRequestPhase.Sending(request), AuthRequestPhase.Waiting(request))) {
                        _state.update { AuthRequestUIState.Error(error.text(context)) }
                    }
                },
            )
        }
    }

    fun onReject() {
        if (phase.value is AuthRequestPhase.Signing || phase.value is AuthRequestPhase.Sending) {
            return
        }
        rejectOnce()
        finish()
    }

    override fun onCleared() {
        rejectOnce()
    }

    private fun rejectOnce() {
        val request = phase.value.request ?: return
        if (answer(request) || phase.compareAndSet(AuthRequestPhase.Signing(request), AuthRequestPhase.Answered(request))) {
            approveWalletConnectAuthentication.rejectAuthentication(request)
        }
    }

    private fun isWaiting(request: GemWalletConnectAuthentication): Boolean = phase.value == AuthRequestPhase.Waiting(request)

    private fun answer(request: GemWalletConnectAuthentication): Boolean = phase.compareAndSet(AuthRequestPhase.Waiting(request), AuthRequestPhase.Answered(request))

    private fun finish(request: GemWalletConnectAuthentication) {
        if (activeRequest.finish(request)) {
            reset()
        }
    }

    private fun finish() {
        reset()
        activeRequest.finish()
    }

    private fun reset() {
        phase.value = AuthRequestPhase.Idle
        _state.update { AuthRequestUIState.Loading }
    }

    private suspend fun requestContent(request: GemWalletConnectAuthentication): AuthRequestUIState.Request {
        val prepared = walletConnectService.prepareAuthentication(
            chainIds = request.chainIds,
            domain = request.domain,
            metadata = request.metadata,
            origin = request.origin,
            validation = request.validation,
        )
        val selectedWallet = prepared.proposal.defaultWallet.toPrimitives()
        val availableWallets = prepared.proposal.wallets.map { it.toPrimitives() }
        return AuthRequestUIState.Request(
            texts = ReviewTexts(context),
            peer = applicationConnectionRow(prepared.proposal.metadata),
            availableWallets = availableWallets,
            availableWalletSections = walletSections(availableWallets.map { it.listItem.toGem() }, null),
            canChooseWallet = prepared.canChooseWallet,
            selectedWallet = selectedWallet,
            approval = buildApproval(request, selectedWallet),
        )
    }

    private fun buildApproval(request: GemWalletConnectAuthentication, wallet: Wallet): AuthApproval {
        val authMessage = approveWalletConnectAuthentication.authMessage(request, wallet)
        val account = authMessage.account.toPrimitives()
        val message = authMessage.message
        val signMessage = SignMessage(
            chain = account.chain.string,
            signType = SignDigestType.SIWE,
            data = message.toByteArray(),
        )
        val payloadPreview = payloadPreview(signMessage)

        return AuthApproval(
            wallet = wallet,
            account = account,
            message = message,
            signMessage = signMessage,
            title = payloadPreview.title,
            primaryPayloadFields = payloadPreview.primaryFields,
            secondaryPayloadFields = payloadPreview.secondaryFields,
        )
    }

    private fun payloadPreview(signMessage: SignMessage): AuthPayloadPreview = signMessageService.payloadPreview(signMessage)?.let { preview ->
        AuthPayloadPreview(
            title = preview.title,
            primaryFields = preview.primary,
            secondaryFields = preview.secondary,
        )
    } ?: AuthPayloadPreview()
}

private sealed interface AuthRequestPhase {
    val request: GemWalletConnectAuthentication?

    data object Idle : AuthRequestPhase {
        override val request: GemWalletConnectAuthentication? = null
    }

    data class Waiting(override val request: GemWalletConnectAuthentication) : AuthRequestPhase

    data class Signing(override val request: GemWalletConnectAuthentication) : AuthRequestPhase

    data class Sending(override val request: GemWalletConnectAuthentication) : AuthRequestPhase

    data class Answered(override val request: GemWalletConnectAuthentication) : AuthRequestPhase
}

sealed interface AuthRequestUIState {

    data object Loading : AuthRequestUIState

    class Error(val text: String) : AuthRequestUIState

    sealed interface Content :
        AuthRequestUIState,
        WalletConnectReviewModel {
        val peer: GemConnectionRow
        val availableWallets: List<Wallet>
        val availableWalletSections: List<GemWalletSection>
        val canChooseWallet: Boolean
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
        override val primaryPayloadFields: List<GemListRow> get() = approval.primaryPayloadFields
        override val secondaryPayloadFields: List<GemListRow> get() = approval.secondaryPayloadFields
        override val title: GemLocalizedText get() = approval.title
        override val message: String get() = approval.message
    }

    data class Request(
        override val peer: GemConnectionRow,
        override val availableWallets: List<Wallet>,
        override val availableWalletSections: List<GemWalletSection>,
        override val canChooseWallet: Boolean,
        override val selectedWallet: Wallet,
        override val approval: AuthApproval,
        override val texts: ReviewTexts,
    ) : Content

    data class Approving(private val request: Request) : Content {
        override val peer: GemConnectionRow get() = request.peer
        override val availableWallets: List<Wallet> get() = request.availableWallets
        override val availableWalletSections: List<GemWalletSection> get() = request.availableWalletSections
        override val canChooseWallet: Boolean get() = request.canChooseWallet
        override val selectedWallet: Wallet get() = request.selectedWallet
        override val approval: AuthApproval get() = request.approval
        override val texts: ReviewTexts get() = request.texts
    }
}

data class AuthApproval(val wallet: Wallet, val account: Account, val message: String, val signMessage: SignMessage, val title: GemLocalizedText, val primaryPayloadFields: List<GemListRow>, val secondaryPayloadFields: List<GemListRow>) {
    val chain: Chain get() = account.chain
}

private data class AuthPayloadPreview(val title: GemLocalizedText = GemLocalizedText.ReviewRequest, val primaryFields: List<GemListRow> = emptyList(), val secondaryFields: List<GemListRow> = emptyList())
