package com.gemwallet.android.features.setup_wallet.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet.coordinators.SetWalletName
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType
import dagger.assisted.Assisted
import dagger.assisted.AssistedFactory
import dagger.assisted.AssistedInject
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

@HiltViewModel(assistedFactory = SetupWalletViewModel.Factory::class)
class SetupWalletViewModel @AssistedInject constructor(
    @Assisted private val walletId: WalletId,
    private val walletsRepository: WalletsRepository,
    private val setWalletName: SetWalletName,
) : ViewModel() {

    private val state = MutableStateFlow(SetupWalletViewModelState())
    val uiState = state.map { it }
        .stateIn(viewModelScope, SharingStarted.Eagerly, SetupWalletViewModelState())

    init {
        viewModelScope.launch {
            walletsRepository.getWallet(walletId).collect { wallet ->
                if (wallet != null) {
                    state.update {
                        it.copy(
                            walletName = wallet.name,
                            walletSource = wallet.source,
                            walletType = wallet.type,
                            walletChain = wallet.accounts.firstOrNull()?.chain,
                            imageUrl = wallet.imageUrl,
                        )
                    }
                }
            }
        }
    }

    fun onNameChange(name: String) {
        state.update { it.copy(walletName = name) }
        viewModelScope.launch {
            setWalletName.setWalletName(walletId, name)
        }
    }

    @AssistedFactory
    interface Factory {
        fun create(walletId: WalletId): SetupWalletViewModel
    }
}

data class SetupWalletViewModelState(
    val walletName: String = "",
    val walletSource: WalletSource = WalletSource.Create,
    val walletType: WalletType? = null,
    val walletChain: Chain? = null,
    val imageUrl: String? = null,
)
