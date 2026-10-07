package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ui.models.navigation.requireChain
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import uniffi.gemstone.GemWalletSecret
import uniffi.gemstone.GemWalletSecretKind
import uniffi.gemstone.GemWalletServiceInterface
import javax.inject.Inject

@HiltViewModel
class SecretDataViewModel @Inject constructor(private val service: GemWalletServiceInterface, savedStateHandle: SavedStateHandle, @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher) : ViewModel() {
    val secretKind = savedStateHandle.requireSecretKind()

    val secret = MutableStateFlow<Result<GemWalletSecret>?>(null)

    init {
        val walletId = savedStateHandle.requireWalletId()
        val chain = when (secretKind) {
            GemWalletSecretKind.PHRASE -> null
            GemWalletSecretKind.PRIVATE_KEY -> savedStateHandle.requireChain()
        }
        viewModelScope.launch(ioDispatcher) {
            secret.value = runCatchingCancellable {
                when (chain) {
                    null -> service.exportSecret(walletId.id)
                    else -> service.exportPrivateKey(walletId.id, chain.string)
                }
            }
        }
    }
}
