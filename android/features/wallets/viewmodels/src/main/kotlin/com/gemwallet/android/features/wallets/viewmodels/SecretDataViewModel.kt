package com.gemwallet.android.features.wallets.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.features.wallets.viewmodels.models.WalletSecretContentUIModel
import com.gemwallet.android.features.wallets.viewmodels.models.uiModel
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import uniffi.gemstone.GemWalletServiceInterface
import javax.inject.Inject

@HiltViewModel
class SecretDataViewModel @Inject constructor(private val service: GemWalletServiceInterface, savedStateHandle: SavedStateHandle, @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher) : ViewModel() {
    val secretKind = savedStateHandle.requireSecretKind()

    val secret = MutableStateFlow<Result<WalletSecretContentUIModel>?>(null)

    init {
        val walletId = savedStateHandle.requireWalletId()
        viewModelScope.launch(ioDispatcher) {
            secret.value = runCatchingCancellable { service.exportSecret(walletId.id).uiModel() }
        }
    }
}
