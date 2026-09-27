package com.gemwallet.android.features.wallets.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemWalletDeletion
import uniffi.gemstone.GemWalletSection
import uniffi.gemstone.GemWalletServiceInterface
import javax.inject.Inject

@HiltViewModel
class WalletsViewModel @Inject constructor(
    private val getAllWallets: GetAllWallets,
    private val service: GemWalletServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val sections: StateFlow<List<GemWalletSection>> = getAllWallets.getAllWallets()

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

    fun selectWallet(walletId: WalletId, onSelected: () -> Unit) = viewModelScope.launch {
        runCatchingCancellable { withContext(ioDispatcher) { service.setCurrentWalletId(walletId.id) } }
            .onSuccess { onSelected() }
            .onFailure(::showError)
    }

    fun deleteWallet(walletId: WalletId, onBoard: () -> Unit) = viewModelScope.launch {
        runCatchingCancellable {
            when (withContext(ioDispatcher) { service.deleteWallet(walletId.id) }) {
                GemWalletDeletion.WALLETS_REMAINING -> Unit
                GemWalletDeletion.LAST_WALLET_DELETED -> onBoard()
            }
        }
            .onFailure(::showError)
    }

    fun togglePin(walletId: WalletId) = viewModelScope.launch(ioDispatcher) {
        val row = sections.value.flatMap { it.rows }.firstOrNull { it.id == walletId.id } ?: return@launch
        runCatchingCancellable { service.setPinned(walletId.id, !row.isPinned) }
            .onFailure(::showError)
    }

    fun clearError() = errorState.update { null }

    private fun showError(error: Throwable) {
        errorState.value = error.errorText().text(context)
    }
}
