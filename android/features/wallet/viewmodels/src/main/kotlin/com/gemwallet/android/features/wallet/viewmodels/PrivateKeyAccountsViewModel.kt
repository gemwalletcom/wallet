package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.ext.runCatchingCancellable
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import uniffi.gemstone.GemPrivateKeyAccount
import uniffi.gemstone.GemWalletServiceInterface
import javax.inject.Inject

@HiltViewModel
class PrivateKeyAccountsViewModel @Inject constructor(private val service: GemWalletServiceInterface, savedStateHandle: SavedStateHandle, @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher) : ViewModel() {
    private val accountsState = MutableStateFlow<Result<List<GemPrivateKeyAccount>>?>(null)
    val accounts: StateFlow<Result<List<GemPrivateKeyAccount>>?> = accountsState.asStateFlow()

    init {
        val walletId = savedStateHandle.requireWalletId()
        viewModelScope.launch(ioDispatcher) {
            accountsState.value = runCatchingCancellable { service.privateKeyAccounts(walletId.id) }
        }
    }
}
