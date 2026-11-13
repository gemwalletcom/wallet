package com.gemwallet.android.features.wallet_connector.viewmodels

import android.content.Context
import android.util.Log
import androidx.compose.runtime.Immutable
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnection
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.listItem
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.wallet.core.primitives.WalletId
import dagger.assisted.Assisted
import dagger.assisted.AssistedFactory
import dagger.assisted.AssistedInject
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemSessionProposal
import uniffi.gemstone.GemWalletConnectException
import uniffi.gemstone.GemWalletConnectProposal
import uniffi.gemstone.GemWalletConnectRejectionReason
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.WalletConnectionVerificationStatus
import uniffi.gemstone.applicationConnectionRow
import uniffi.gemstone.connectionProposal
import uniffi.gemstone.walletSections

@HiltViewModel(assistedFactory = ConnectionProposalViewModel.Factory::class)
class ConnectionProposalViewModel @AssistedInject constructor(
    @Assisted private val proposal: GemWalletConnectProposal,
    private val approveWalletConnection: ApproveWalletConnection,
    private val activeRequest: ActiveWalletConnectRequest,
    private val walletConnectService: GemWalletConnectServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val state = MutableStateFlow<ConnectionProposalUIState>(ConnectionProposalUIState.Init(WalletConnectionVerificationStatus.UNKNOWN))

    private val refusals = Channel<String>(Channel.BUFFERED)
    val refusalMessages: Flow<String> = refusals.receiveAsFlow()

    private val _sessionProposal = MutableStateFlow<GemSessionProposal?>(null)

    val peer = _sessionProposal.map { prepared -> prepared?.let { applicationConnectionRow(it.proposal.metadata) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val availableWallets = _sessionProposal.map { prepared -> prepared?.proposal?.wallets.orEmpty().map { it.toPrimitives() } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val availableWalletSections = availableWallets.map { wallets -> walletSections(wallets.map { it.listItem.toGem() }, null) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val _selectedWallet = MutableStateFlow<com.wallet.core.primitives.Wallet?>(null)

    val selectedWallet = combine(_selectedWallet, _sessionProposal) { wallet, prepared ->
        wallet ?: prepared?.proposal?.defaultWallet?.toPrimitives()
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val proposalRows = combine(_sessionProposal, _selectedWallet) { prepared, wallet ->
        prepared?.let { connectionProposal(it.proposal, wallet?.toGem()?.id, it.verificationStatus) }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val buttonState = combine(selectedWallet, state) { wallet, sceneState ->
        buttonState(enabled = wallet != null, loading = sceneState is ConnectionProposalUIState.Approving)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Disabled)

    init {
        viewModelScope.launch {
            val prepared = withContext(ioDispatcher) {
                runCatchingCancellable {
                    walletConnectService.prepareSessionProposal(
                        requiredChainIds = proposal.requiredChainIds,
                        optionalChainIds = proposal.optionalChainIds,
                        metadata = proposal.metadata,
                        origin = proposal.origin,
                        validation = proposal.validation,
                    )
                }
            }.getOrElse { error ->
                Log.e(TAG, "session proposal rejected: ${error.message}")
                refusals.trySend(error.errorText().text(context))
                reject((error as? GemWalletConnectException)?.rejectionReason() ?: GemWalletConnectRejectionReason.USER_REJECTED)
                return@launch
            }
            if (state.value !is ConnectionProposalUIState.Init) {
                return@launch
            }
            state.update { ConnectionProposalUIState.Init(prepared.verificationStatus) }
            _sessionProposal.update { prepared }
        }
    }

    fun onApprove(onError: (GemErrorText) -> Unit) {
        val wallet = selectedWallet.value ?: return
        if (state.value !is ConnectionProposalUIState.Init) {
            return
        }
        state.update { ConnectionProposalUIState.Approving(it.verificationStatus) }
        approveWalletConnection.approveConnection(
            wallet = wallet,
            proposal = proposal,
            onSuccess = ::finish,
            onError = { error -> fail(error, onError) },
        )
    }

    fun onReject() {
        reject()
    }

    fun onWalletSelected(walletId: WalletId) {
        if (state.value is ConnectionProposalUIState.Approving) {
            return
        }
        _selectedWallet.update { availableWallets.value.firstOrNull { it.id == walletId } }
    }

    override fun onCleared() {
        onReject()
    }

    private fun reject(reason: GemWalletConnectRejectionReason = GemWalletConnectRejectionReason.USER_REJECTED) {
        if (state.value !is ConnectionProposalUIState.Init) {
            return
        }
        state.update { ConnectionProposalUIState.Rejected(it.verificationStatus) }
        approveWalletConnection.rejectConnection(
            proposal = proposal,
            reason = reason,
            onSuccess = ::finish,
            onError = { finish() },
        )
    }

    private fun fail(error: GemErrorText, onError: (GemErrorText) -> Unit) {
        if (activeRequest.finish(proposal)) {
            onError(error)
        }
    }

    private fun finish() {
        activeRequest.finish(proposal)
    }

    @AssistedFactory
    interface Factory {
        fun create(proposal: GemWalletConnectProposal): ConnectionProposalViewModel
    }

    private companion object {
        const val TAG = "ConnectionProposalViewModel"
    }
}

@Immutable
sealed interface ConnectionProposalUIState {
    val verificationStatus: WalletConnectionVerificationStatus

    data class Init(override val verificationStatus: WalletConnectionVerificationStatus) : ConnectionProposalUIState

    data class Approving(override val verificationStatus: WalletConnectionVerificationStatus) : ConnectionProposalUIState

    data class Rejected(override val verificationStatus: WalletConnectionVerificationStatus) : ConnectionProposalUIState
}
