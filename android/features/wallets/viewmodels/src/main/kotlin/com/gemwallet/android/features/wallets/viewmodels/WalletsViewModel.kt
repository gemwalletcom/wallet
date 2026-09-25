package com.gemwallet.android.features.wallets.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.domains.wallet.aggregates.WalletDataAggregate
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.features.wallets.viewmodels.models.WalletItemUIModel
import com.gemwallet.android.features.wallets.viewmodels.models.WalletsUIState
import com.gemwallet.android.ui.components.list_item.uiModel
import com.gemwallet.android.ui.localization.text
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
import uniffi.gemstone.GemWalletDeletion
import uniffi.gemstone.GemWalletServiceInterface
import javax.inject.Inject

@HiltViewModel
class WalletsViewModel @Inject constructor(
    private val getAllWallets: GetAllWallets,
    private val service: GemWalletServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val wallets = getAllWallets.getAllWallets()

    val uiState: StateFlow<WalletsUIState> = wallets.map { wallets ->
        val (pinned, unpinned) = wallets.partition { it.row.isPinned }
        WalletsUIState(pinned = pinned.map { it.uiModel(context) }, unpinned = unpinned.map { it.uiModel(context) })
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, WalletsUIState())

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
        val wallet = wallets.value.firstOrNull { it.row.id == walletId.id } ?: return@launch
        runCatchingCancellable { service.setPinned(walletId.id, !wallet.row.isPinned) }
            .onFailure(::showError)
    }

    fun clearError() = errorState.update { null }

    private fun showError(error: Throwable) {
        errorState.value = error.errorText().text(context)
    }
}

private fun WalletDataAggregate.uiModel(context: Context) = WalletItemUIModel(row = row.uiModel(context), isCurrent = isCurrent)
