package com.gemwallet.android.features.bridge.viewmodels

import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_connect.coordinators.PrepareSessionProposal
import com.gemwallet.android.data.repositories.bridge.ActiveWalletConnectRequest
import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.data.repositories.bridge.WalletConnectSessionProposal
import com.gemwallet.android.data.repositories.bridge.WalletConnectVerifyContext
import com.wallet.core.primitives.WalletConnectionSessionProposal
import com.gemwallet.android.features.bridge.viewmodels.model.BridgeRequestError
import com.gemwallet.android.features.bridge.viewmodels.model.WalletConnectOriginVerifier
import com.gemwallet.android.features.bridge.viewmodels.model.toSessionUI
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.WalletConnectionVerificationStatus
import javax.inject.Inject

@HiltViewModel
class ProposalSceneViewModel @Inject constructor(
    private val bridgesRepository: BridgesRepository,
    private val prepareSessionProposal: PrepareSessionProposal,
    private val originVerifier: WalletConnectOriginVerifier,
    private val activeRequest: ActiveWalletConnectRequest,
) : ViewModel() {

    val state = MutableStateFlow<ProposalSceneState>(ProposalSceneState.Init(WalletConnectionVerificationStatus.UNKNOWN))

    private val _proposal = MutableStateFlow<WalletConnectSessionProposal?>(null)
    private val _sessionProposal = MutableStateFlow<WalletConnectionSessionProposal?>(null)

    val proposal = _sessionProposal.map { it?.metadata?.toSessionUI() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val availableWallets = _sessionProposal.map { it?.wallets.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val _selectedWallet = MutableStateFlow<com.wallet.core.primitives.Wallet?>(null)

    val selectedWallet = combine(_selectedWallet, _sessionProposal) { wallet, proposal ->
        wallet ?: proposal?.defaultWallet
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val buttonState = combine(selectedWallet, state) { wallet, sceneState ->
        buttonState(enabled = wallet != null, loading = sceneState is ProposalSceneState.Approving)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Disabled)

    fun onProposal(
        proposal: WalletConnectSessionProposal,
        verifyContext: WalletConnectVerifyContext,
        onNotify: (BridgeRequestError) -> Unit,
    ) {
        val verification = originVerifier.verify(proposal.url, verifyContext)
        if (verification.isScam) {
            onNotify(BridgeRequestError.MaliciousSession)
            reject(proposal)
            return
        }
        viewModelScope.launch(Dispatchers.IO) {
            val prepared = runCatchingCancellable {
                prepareSessionProposal(
                    name = proposal.name,
                    description = proposal.description,
                    url = proposal.url,
                    icons = proposal.icons,
                    requiredChainIds = proposal.requiredNamespaces.values.flatMap { it.chains.orEmpty() },
                    optionalChainIds = proposal.optionalNamespaces.values.flatMap { it.chains.orEmpty() },
                    origin = verifyContext.origin,
                    validation = verification.status,
                )
            }.getOrElse { error ->
                Log.e(TAG, "session proposal rejected: ${error.message}")
                reject(proposal)
                return@launch
            }
            state.update { ProposalSceneState.Init(prepared.verificationStatus) }
            _sessionProposal.update { prepared.proposal }
            _proposal.update { proposal }
        }
    }

    fun onApprove(onError: (String) -> Unit) {
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
        viewModelScope.launch(Dispatchers.IO) {
            val result = runCatching {
                bridgesRepository.approveConnection(
                    wallet = wallet,
                    proposal = proposal,
                    onSuccess = { finish(proposal) },
                    onError = { message -> fail(proposal, message, onError) }
                )
            }
            result.onFailure { err -> fail(proposal, err.message.orEmpty(), onError) }
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

    private fun reject(proposal: WalletConnectSessionProposal) {
        viewModelScope.launch(Dispatchers.IO) {
            bridgesRepository.rejectConnection(
                proposal = proposal,
                onSuccess = { finish(proposal) },
                onError = { finish(proposal) }
            )
        }
    }

    private fun fail(proposal: WalletConnectSessionProposal, message: String, onError: (String) -> Unit) {
        if (activeRequest.finish(proposal)) {
            reset()
            onError(message)
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

    data class Init(
        override val verificationStatus: WalletConnectionVerificationStatus,
    ) : ProposalSceneState

    data class Approving(
        override val verificationStatus: WalletConnectionVerificationStatus,
    ) : ProposalSceneState

}
