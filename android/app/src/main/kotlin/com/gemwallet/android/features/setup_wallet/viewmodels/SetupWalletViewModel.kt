package com.gemwallet.android.features.setup_wallet.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.wallet.cases.GetWallet
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSource
import dagger.assisted.Assisted
import dagger.assisted.AssistedFactory
import dagger.assisted.AssistedInject
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.GemWalletServiceInterface
import uniffi.gemstone.walletRow

@HiltViewModel(assistedFactory = SetupWalletViewModel.Factory::class)
class SetupWalletViewModel @AssistedInject constructor(
    @Assisted private val walletId: WalletId,
    private val getWallet: GetWallet,
    private val service: GemWalletServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val state = MutableStateFlow(SetupWalletViewModelState())
    val uiState = state.map { it }
        .stateIn(viewModelScope, SharingStarted.Eagerly, SetupWalletViewModelState())

    init {
        viewModelScope.launch {
            getWallet(walletId).collect { wallet ->
                if (wallet != null) {
                    state.update {
                        it.copy(
                            walletName = wallet.name,
                            walletSource = wallet.source,
                            row = walletRow(wallet.toGem()),
                        )
                    }
                }
            }
        }
    }

    fun onNameChange(name: String) {
        state.update { it.copy(walletName = name) }
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { service.rename(walletId.id, name) }
                .onFailure { error -> state.update { it.copy(error = error.errorText().text(context)) } }
        }
    }

    fun clearError() = state.update { it.copy(error = null) }

    @AssistedFactory
    interface Factory {
        fun create(walletId: WalletId): SetupWalletViewModel
    }
}

data class SetupWalletViewModelState(val walletName: String = "", val walletSource: WalletSource = WalletSource.Create, val row: GemWalletRow? = null, val error: String? = null)
