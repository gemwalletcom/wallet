package com.gemwallet.android.features.bridge.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionProposal
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnection
import com.gemwallet.android.application.wallet_connect.cases.PrepareSessionProposal
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.bridge.viewmodels.localization.text
import com.gemwallet.android.features.bridge.viewmodels.model.ConnectionHeadUIModel
import com.gemwallet.android.features.bridge.viewmodels.model.headUIModel
import com.gemwallet.android.features.bridge.viewmodels.model.map
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemSymbol
import com.gemwallet.android.ui.components.list_item.uiModel
import com.gemwallet.android.ui.localization.titleRes
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.style.textStyle
import com.wallet.core.primitives.WalletConnectionSessionProposal
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemApplicationMetadataServiceInterface
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemWalletConnectException
import uniffi.gemstone.GemWalletConnectFailure
import uniffi.gemstone.GemWalletConnectRejectionReason
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.WalletConnectionVerificationStatus
import uniffi.gemstone.walletRows
import javax.inject.Inject

@HiltViewModel
class ProposalSceneViewModel @Inject constructor(
    private val approveWalletConnection: ApproveWalletConnection,
    private val prepareSessionProposal: PrepareSessionProposal,
    private val activeRequest: ActiveWalletConnectRequest,
    private val walletConnectService: GemWalletConnectServiceInterface,
    private val metadataService: GemApplicationMetadataServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val state = MutableStateFlow<ProposalSceneState>(ProposalSceneState.Init(WalletConnectionVerificationStatus.UNKNOWN))

    private val _proposal = MutableStateFlow<WalletConnectSessionProposal?>(null)
    private val _sessionProposal = MutableStateFlow<WalletConnectionSessionProposal?>(null)

    val proposal = _sessionProposal.map { proposal -> proposal?.metadata?.let { metadataService.connectionRow(it.toGem()) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val peerHead: StateFlow<ConnectionHeadUIModel?> = proposal.map { it?.headUIModel() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val availableWallets = _sessionProposal.map { it?.wallets.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val availableWalletRows = availableWallets.map { wallets -> walletRows(wallets.map { it.toGem() }).map { it.uiModel(context) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val _selectedWallet = MutableStateFlow<com.wallet.core.primitives.Wallet?>(null)

    val selectedWallet = combine(_selectedWallet, _sessionProposal) { wallet, proposal ->
        wallet ?: proposal?.defaultWallet
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val walletListItem = selectedWallet.map { ListItemModel(title = context.getString(R.string.common_wallet), subtitle = it?.name.orEmpty()) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ListItemModel(title = context.getString(R.string.common_wallet), subtitle = ""))

    val connectionListItem = ListItemModel(title = context.getString(R.string.wallet_connect_connection_title), subtitle = context.getString(R.string.wallet_connect_brand_name))

    val statusListItem = state.map { sceneState ->
        ListItemModel(
            title = context.getString(R.string.transaction_status),
            subtitle = context.getString(sceneState.verificationStatus.titleRes()),
            subtitleStyle = sceneState.verificationStatus.textStyle(),
        )
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ListItemModel(title = context.getString(R.string.transaction_status)))

    val permissionListItems = listOf(R.string.wallet_connect_permissions_view_balance, R.string.wallet_connect_permissions_approval_requests)
        .map { ListItemModel(title = context.getString(it), image = ListItemImage.Symbol(ListItemSymbol.Check)) }

    val buttonState = combine(selectedWallet, state) { wallet, sceneState ->
        buttonState(enabled = wallet != null, loading = sceneState is ProposalSceneState.Approving)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Disabled)

    fun onProposal(proposal: WalletConnectSessionProposal, verifyContext: WalletConnectVerifyContext, onNotify: (String) -> Unit) {
        if (!walletConnectService.shouldProcessMessage("proposal_${proposal.proposerPublicKey}")) {
            return
        }
        viewModelScope.launch {
            val prepared = withContext(ioDispatcher) {
                runCatchingCancellable {
                    prepareSessionProposal(
                        name = proposal.name,
                        description = proposal.description,
                        url = proposal.url,
                        icons = proposal.icons,
                        requiredChainIds = proposal.requiredNamespaces.values.flatMap { it.chains.orEmpty() },
                        optionalChainIds = proposal.optionalNamespaces.values.flatMap { it.chains.orEmpty() },
                        origin = verifyContext.origin,
                        validation = verifyContext.map(),
                    )
                }
            }.getOrElse { error ->
                Log.e(TAG, "session proposal rejected: ${error.message}")
                if (error is GemWalletConnectException.InvalidOrigin) onNotify(GemWalletConnectFailure.MaliciousOrigin.text(context))
                reject(proposal, (error as? GemWalletConnectException)?.rejectionReason() ?: GemWalletConnectRejectionReason.USER_REJECTED)
                return@launch
            }
            state.update { ProposalSceneState.Init(prepared.verificationStatus) }
            _sessionProposal.update { prepared.proposal }
            _proposal.update { proposal }
        }
    }

    fun onApprove(onError: (GemErrorText) -> Unit) {
        val wallet = selectedWallet.value
        val proposal = _proposal.value
        if (state.value is ProposalSceneState.Approving) {
            return
        }

        if (wallet == null || proposal == null) {
            finish()
            return
        }
        state.update { ProposalSceneState.Approving(it.verificationStatus) }
        viewModelScope.launch(ioDispatcher) {
            val result = runCatching {
                approveWalletConnection.approveConnection(
                    wallet = wallet,
                    proposal = proposal,
                    onSuccess = { finish(proposal) },
                    onError = { message -> fail(proposal, GemErrorText.Message(message), onError) },
                )
            }
            result.onFailure { err -> fail(proposal, err.errorText(), onError) }
        }
    }

    fun onReject() {
        if (state.value is ProposalSceneState.Approving) {
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
        if (state.value is ProposalSceneState.Approving) {
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
        state.update { ProposalSceneState.Init(WalletConnectionVerificationStatus.UNKNOWN) }
    }

    private companion object {
        const val TAG = "ProposalSceneViewModel"
    }
}

sealed interface ProposalSceneState {
    val verificationStatus: WalletConnectionVerificationStatus

    data class Init(override val verificationStatus: WalletConnectionVerificationStatus) : ProposalSceneState

    data class Approving(override val verificationStatus: WalletConnectionVerificationStatus) : ProposalSceneState
}
