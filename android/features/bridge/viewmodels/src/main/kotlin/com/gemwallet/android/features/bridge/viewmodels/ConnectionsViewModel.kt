package com.gemwallet.android.features.bridge.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_connect.cases.GetWalletConnections
import com.gemwallet.android.application.wallet_connect.cases.PairWalletConnect
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class ConnectionsViewModel @Inject constructor(
    private val getWalletConnections: GetWalletConnections,
    private val pairWalletConnect: PairWalletConnect,
) : ViewModel() {

    val connections = getWalletConnections.observeConnections()
        .stateIn(viewModelScope, SharingStarted.Companion.Lazily, emptyList())

    fun addPairing(uri: String, onSuccess: () -> Unit, onError: (String) -> Unit) {
        viewModelScope.launch(Dispatchers.IO) {
            pairWalletConnect.pair(
                uri = uri,
                onSuccess = onSuccess,
                onError = onError,
            )
        }
    }
}