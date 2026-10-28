package com.gemwallet.android.features.wallet.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ui.localization.text
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
import uniffi.gemstone.GemWalletRow
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
    private val _isSelected = MutableStateFlow(false)
    val isSelected: StateFlow<Boolean> = _isSelected.asStateFlow()
    private val _deletion = MutableStateFlow<GemWalletDeletion?>(null)
    val deletion: StateFlow<GemWalletDeletion?> = _deletion.asStateFlow()

    fun selectWallet(row: GemWalletRow) = viewModelScope.launch {
        runCatchingCancellable { withContext(ioDispatcher) { service.setCurrentWalletId(row.id) } }
            .onSuccess { _isSelected.value = true }
            .onFailure(::showError)
    }

    fun onSelectedHandled() {
        _isSelected.value = false
    }

    fun deleteWallet(row: GemWalletRow) = viewModelScope.launch {
        runCatchingCancellable { _deletion.value = withContext(ioDispatcher) { service.deleteWallet(row.id) } }
            .onFailure(::showError)
    }

    fun onDeletionHandled() {
        _deletion.value = null
    }

    fun togglePin(row: GemWalletRow) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { service.setPinned(row.id, !row.isPinned) }
            .onFailure(::showError)
    }

    fun clearError() = errorState.update { null }

    private fun showError(error: Throwable) {
        errorState.value = error.errorText().text(context)
    }
}
