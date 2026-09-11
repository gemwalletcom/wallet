package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.ext.runCatchingCancellable
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import uniffi.gemstone.GemWalletSecret
import uniffi.gemstone.GemWalletServiceInterface
import javax.inject.Inject

@HiltViewModel
class WalletSecretDataViewModel @Inject constructor(
    private val service: GemWalletServiceInterface,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    val secretKind = savedStateHandle.requireSecretKind()
    val chain = savedStateHandle.chain()

    val secret = MutableStateFlow<Result<GemWalletSecret>?>(null)

    init {
        val walletId = savedStateHandle.requireWalletId()
        viewModelScope.launch(Dispatchers.IO) {
            secret.value = runCatchingCancellable {
                when (chain) {
                    null -> service.exportSecret(walletId.id)
                    else -> GemWalletSecret.PrivateKey(chain.string, service.exportPrivateKey(walletId.id, chain.string))
                }
            }
        }
    }
}
