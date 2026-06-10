package com.gemwallet.android.features.bridge.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.ext.walletConnectAppName
import com.gemwallet.android.ext.walletConnectIcon
import com.gemwallet.android.features.bridge.viewmodels.model.WalletConnectOriginVerifier
import com.gemwallet.android.features.bridge.viewmodels.model.toSessionUI
import com.reown.walletkit.client.Wallet
import com.wallet.core.primitives.WalletConnectionSessionAppMetadata
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.WalletConnectionVerificationStatus
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ProposalSceneViewModel @Inject constructor(
    sessionRepository: SessionRepository,
    private val bridgesRepository: BridgesRepository,
    private val walletsRepository: WalletsRepository,
    private val originVerifier: WalletConnectOriginVerifier,
) : ViewModel() {

    val state = MutableStateFlow<ProposalSceneState>(ProposalSceneState.Init(WalletConnectionVerificationStatus.UNKNOWN))

    private val _proposal = MutableStateFlow<Wallet.Model.SessionProposal?>(null)

    val proposal = _proposal.map {
        it ?: return@map null
        val icons = it.icons.map { it.toString() }
        WalletConnectionSessionAppMetadata(
            name = walletConnectAppName(it.name, it.url),
            description = it.description,
            url = it.url,
            icon = icons.walletConnectIcon(),
        ).toSessionUI()
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val availableWallets = _proposal.filterNotNull().mapLatest { proposal ->
        val chains = proposal.supportedWalletConnectProposalChains() ?: return@mapLatest emptyList()
        (walletsRepository.getAll().firstOrNull() ?: emptyList())
            .walletsSupportingWalletConnectProposal(chains)
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val _selectedWallet = MutableStateFlow<com.wallet.core.primitives.Wallet?>(null)
    val selectedWallet = combine(
        _selectedWallet,
        sessionRepository.session(),
        availableWallets,
    ) { wallet, session, availableWallets ->
        val current = session?.wallet
        wallet ?: availableWallets.firstOrNull { current?.id == it.id } ?: availableWallets.firstOrNull()
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)


    fun onProposal(
        proposal: Wallet.Model.SessionProposal,
        verifyContext: Wallet.Model.VerifyContext
    ) {
        val verification = originVerifier.verify(proposal.url, verifyContext)
        if (verification.isScam) {
            onReject(proposal, ProposalSceneState.ScamCanceled)
            return
        }
        state.update { ProposalSceneState.Init(verification.status) }
        _proposal.update { proposal }
    }

    fun onApprove() {
        val wallet = selectedWallet.value
        val proposal = _proposal.value
        val currentState = state.value as? ProposalSceneState.Content ?: return
        if (currentState is ProposalSceneState.Approving) {
            return
        }

        if (wallet == null || proposal == null) {
            state.update { ProposalSceneState.Canceled }
            return
        }
        state.update { ProposalSceneState.Approving(currentState.verificationStatus) }
        viewModelScope.launch(Dispatchers.IO) {
            val result = runCatching {
                bridgesRepository.approveConnection(
                    wallet = wallet,
                    proposal = proposal,
                    onSuccess = { state.update { ProposalSceneState.Canceled } },
                    onError = { message -> state.update { ProposalSceneState.Fail(message) } }
                )
            }
            result.onFailure { err -> state.update { ProposalSceneState.Fail(err.message ?: "Connection failed") } }
        }
    }

    fun onReject(){
        if (state.value is ProposalSceneState.Approving) {
            return
        }
        onReject(_proposal.value ?: return)
    }

    fun onReject(proposal: Wallet.Model.SessionProposal, withState: ProposalSceneState = ProposalSceneState.Canceled) = viewModelScope.launch(Dispatchers.IO) {
        bridgesRepository.rejectConnection(
            proposal = proposal,
            onSuccess = { state.update { withState } },
            onError = { state.update { withState } }
        )
    }

    fun onWalletSelected(walletId: WalletId) {
        if (state.value is ProposalSceneState.Approving) {
            return
        }
        _selectedWallet.update { availableWallets.value.firstOrNull { it.id == walletId } }
    }

}

sealed interface ProposalSceneState {
    sealed interface Content : ProposalSceneState {
        val verificationStatus: WalletConnectionVerificationStatus
    }

    data class Init(
        override val verificationStatus: WalletConnectionVerificationStatus,
    ) : Content

    data class Approving(
        override val verificationStatus: WalletConnectionVerificationStatus,
    ) : Content

    data object Canceled : ProposalSceneState

    data object ScamCanceled : ProposalSceneState

    class Fail(val message: String) : ProposalSceneState
}
