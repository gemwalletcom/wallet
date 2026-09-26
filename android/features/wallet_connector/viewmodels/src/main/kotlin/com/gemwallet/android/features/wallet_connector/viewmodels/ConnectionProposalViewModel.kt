package com.gemwallet.android.features.wallet_connector.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionProposal
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnection
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.wallet_connector.viewmodels.models.map
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.localization.titleRes
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.style.textStyle
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemSessionProposal
import uniffi.gemstone.GemWalletConnectException
import uniffi.gemstone.GemWalletConnectRejectionReason
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.WalletConnectionVerificationStatus
import uniffi.gemstone.applicationConnectionRow
import uniffi.gemstone.connectionProposal
import uniffi.gemstone.walletSections
import javax.inject.Inject

@HiltViewModel
class ConnectionProposalViewModel @Inject constructor(
    private val approveWalletConnection: ApproveWalletConnection,
    private val activeRequest: ActiveWalletConnectRequest,
    private val walletConnectService: GemWalletConnectServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val state = MutableStateFlow<ConnectionProposalUIState>(ConnectionProposalUIState.Init(WalletConnectionVerificationStatus.UNKNOWN))

    private val _proposal = MutableStateFlow<WalletConnectSessionProposal?>(null)
    private val _sessionProposal = MutableStateFlow<GemSessionProposal?>(null)

    val proposal = _sessionProposal.map { prepared -> prepared?.let { applicationConnectionRow(it.proposal.metadata) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val availableWallets = _sessionProposal.map { prepared -> prepared?.proposal?.wallets.orEmpty().map { it.toPrimitives() } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val availableWalletSections = _sessionProposal.map { prepared -> walletSections(prepared?.proposal?.wallets.orEmpty(), null) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val _selectedWallet = MutableStateFlow<com.wallet.core.primitives.Wallet?>(null)

    val selectedWallet = combine(_selectedWallet, _sessionProposal) { wallet, prepared ->
        wallet ?: prepared?.proposal?.defaultWallet?.toPrimitives()
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val proposalRows = combine(selectedWallet, state) { wallet, sceneState -> connectionProposal(sceneState.verificationStatus, wallet?.name.orEmpty()) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, connectionProposal(WalletConnectionVerificationStatus.UNKNOWN, ""))

    val statusListItem = state.map { sceneState ->
        ListItemModel(
            title = context.getString(R.string.transaction_status),
            subtitle = context.getString(sceneState.verificationStatus.titleRes()),
            subtitleStyle = sceneState.verificationStatus.textStyle(),
        )
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ListItemModel(title = context.getString(R.string.transaction_status)))

    val buttonState = combine(selectedWallet, state) { wallet, sceneState ->
        buttonState(enabled = wallet != null, loading = sceneState is ConnectionProposalUIState.Approving)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Disabled)

    fun onProposal(proposal: WalletConnectSessionProposal, verifyContext: WalletConnectVerifyContext, onNotify: (String) -> Unit) {
        if (!walletConnectService.shouldProcessProposal(proposal.proposerPublicKey)) {
            return
        }
        viewModelScope.launch {
            val prepared = withContext(ioDispatcher) {
                runCatchingCancellable {
                    walletConnectService.prepareSessionProposal(
                        requiredChainIds = proposal.requiredNamespaces.values.flatMap { it.chains.orEmpty() },
                        optionalChainIds = proposal.optionalNamespaces.values.flatMap { it.chains.orEmpty() },
                        metadata = walletConnectService.applicationMetadata(proposal.name, proposal.description, proposal.url, proposal.icons),
                        origin = verifyContext.origin,
                        validation = verifyContext.map(),
                    )
                }
            }.getOrElse { error ->
                Log.e(TAG, "session proposal rejected: ${error.message}")
                onNotify(error.errorText().text(context))
                reject(proposal, (error as? GemWalletConnectException)?.rejectionReason() ?: GemWalletConnectRejectionReason.USER_REJECTED)
                return@launch
            }
            state.update { ConnectionProposalUIState.Init(prepared.verificationStatus) }
            _sessionProposal.update { prepared }
            _proposal.update { proposal }
        }
    }

    fun onApprove(onError: (GemErrorText) -> Unit) {
        val wallet = selectedWallet.value
        val proposal = _proposal.value
        if (state.value is ConnectionProposalUIState.Approving) {
            return
        }

        if (wallet == null || proposal == null) {
            finish()
            return
        }
        state.update { ConnectionProposalUIState.Approving(it.verificationStatus) }
        viewModelScope.launch(ioDispatcher) {
            val result = runCatching {
                approveWalletConnection.approveConnection(
                    wallet = wallet,
                    proposal = proposal,
                    onSuccess = { finish(proposal) },
                    onError = { error -> fail(proposal, error, onError) },
                )
            }
            result.onFailure { err -> fail(proposal, err.errorText(), onError) }
        }
    }

    fun onReject() {
        if (state.value is ConnectionProposalUIState.Approving) {
            return
        }
        val proposal = _proposal.value
        if (proposal == null) {
            finish()
            return
        }
        reject(proposal)
    }

    fun onWalletSelected(walletId: WalletId) {
        if (state.value is ConnectionProposalUIState.Approving) {
            return
        }
        _selectedWallet.update { availableWallets.value.firstOrNull { it.id == walletId } }
    }

    private fun reject(proposal: WalletConnectSessionProposal, reason: GemWalletConnectRejectionReason = GemWalletConnectRejectionReason.USER_REJECTED) {
        viewModelScope.launch(ioDispatcher) {
            approveWalletConnection.rejectConnection(
                proposal = proposal,
                reason = reason,
                onSuccess = { finish(proposal) },
                onError = { finish(proposal) },
            )
        }
    }

    private fun fail(proposal: WalletConnectSessionProposal, error: GemErrorText, onError: (GemErrorText) -> Unit) {
        if (activeRequest.finish(proposal)) {
            reset()
            onError(error)
        }
    }

    private fun finish(proposal: WalletConnectSessionProposal) {
        if (activeRequest.finish(proposal)) {
            reset()
        }
    }

    private fun finish() {
        reset()
        activeRequest.finish()
    }

    private fun reset() {
        _proposal.update { null }
        _sessionProposal.update { null }
        _selectedWallet.update { null }
        state.update { ConnectionProposalUIState.Init(WalletConnectionVerificationStatus.UNKNOWN) }
    }

    private companion object {
        const val TAG = "ConnectionProposalViewModel"
    }
}

sealed interface ConnectionProposalUIState {
    val verificationStatus: WalletConnectionVerificationStatus

    data class Init(override val verificationStatus: WalletConnectionVerificationStatus) : ConnectionProposalUIState

    data class Approving(override val verificationStatus: WalletConnectionVerificationStatus) : ConnectionProposalUIState
}
