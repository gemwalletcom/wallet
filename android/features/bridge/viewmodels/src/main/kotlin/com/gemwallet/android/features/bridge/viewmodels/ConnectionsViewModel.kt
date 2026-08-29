package com.gemwallet.android.features.bridge.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.repositories.bridge.WalletConnectorService
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class ConnectionsViewModel @Inject constructor(
    private val walletConnectorService: WalletConnectorService,
) : ViewModel() {

    val connections = walletConnectorService.getConnections()
        .stateIn(viewModelScope, SharingStarted.Companion.Lazily, emptyList())

    fun addPairing(uri: String, onSuccess: () -> Unit, onError: (String) -> Unit) {
        viewModelScope.launch(Dispatchers.IO) {
            walletConnectorService.addPairing(
                uri = uri,
                onSuccess = onSuccess,
                onError = onError,
            )
        }
    }
}