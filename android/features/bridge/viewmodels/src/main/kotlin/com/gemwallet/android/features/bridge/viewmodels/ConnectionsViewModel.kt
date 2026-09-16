package com.gemwallet.android.features.bridge.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_connect.cases.GetWalletConnections
import com.gemwallet.android.application.wallet_connect.cases.PairWalletConnect
import com.gemwallet.android.data.services.gemstone.di.IoDispatcher
import dagger.hilt.android.lifecycle.HiltViewModel
import com.gemwallet.android.ext.toGem
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.mapLatest
import uniffi.gemstone.GemWalletConnectServiceInterface
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ConnectionsViewModel @Inject constructor(
    getWalletConnections: GetWalletConnections,
    private val pairWalletConnect: PairWalletConnect,
    private val service: GemWalletConnectServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    val sections = getWalletConnections.observeConnections()
        .mapLatest { connections -> service.connectionSections(connections.map { it.toGem() }) }
        .stateIn(viewModelScope, SharingStarted.Companion.Lazily, emptyList())

    fun addPairing(uri: String, onSuccess: () -> Unit, onError: (String) -> Unit) {
        viewModelScope.launch(ioDispatcher) {
            pairWalletConnect.pair(
                uri = uri,
                onSuccess = onSuccess,
                onError = onError,
            )
        }
    }
}