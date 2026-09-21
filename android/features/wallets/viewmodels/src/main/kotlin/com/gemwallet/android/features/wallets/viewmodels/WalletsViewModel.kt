package com.gemwallet.android.features.wallets.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.wallet.cases.DeleteWallet
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemWalletServiceInterface
import javax.inject.Inject

@HiltViewModel
class WalletsViewModel @Inject constructor(
    private val getAllWallets: GetAllWallets,
    private val setCurrentWallet: SetCurrentWallet,
    private val service: GemWalletServiceInterface,
    private val deleteWallet: DeleteWallet,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val wallets = getAllWallets.getAllWallets()
        .stateIn(viewModelScope, SharingStarted.Eagerly, getAllWallets.getAllWallets().value)

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

    fun selectWallet(walletId: WalletId, onSelected: () -> Unit) = viewModelScope.launch {
        runCatchingCancellable { withContext(ioDispatcher) { setCurrentWallet.setCurrentWallet(walletId) } }
            .onSuccess { onSelected() }
            .onFailure(::showError)
    }

    fun deleteWallet(walletId: WalletId, onBoard: () -> Unit) = viewModelScope.launch {
        runCatchingCancellable { deleteWallet.deleteWallet(walletId, onBoard) {} }
            .onFailure(::showError)
    }

    fun togglePin(walletId: WalletId) = viewModelScope.launch(ioDispatcher) {
        val wallet = wallets.value.firstOrNull { it.row.id == walletId.id } ?: return@launch
        runCatchingCancellable { service.setPinned(walletId.id, !wallet.row.isPinned) }
            .onFailure(::showError)
    }

    fun clearError() = errorState.update { null }

    private fun showError(error: Throwable) {
        errorState.value = error.errorText().text(context)
    }
}
