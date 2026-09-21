package com.gemwallet.android.features.bridge.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.wallet_connect.cases.DisconnectWalletConnection
import com.gemwallet.android.application.wallet_connect.cases.GetWalletConnections
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.bridge.viewmodels.model.listItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.models.navigation.RouteArgument
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemWalletConnectServiceInterface
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ConnectionViewModel @Inject constructor(
    getWalletConnections: GetWalletConnections,
    private val disconnectWalletConnection: DisconnectWalletConnection,
    private val service: GemWalletConnectServiceInterface,
    savedState: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val connectionId = savedState.requireString(RouteArgument.ConnectionId)

    val details = getWalletConnections.observeConnection(connectionId)
        .mapLatest { connection -> connection?.let { service.connectionDetails(it.toGem()) } }
        .stateIn(viewModelScope, SharingStarted.Companion.Eagerly, null)

    val connectionListItem: StateFlow<ListItemModel?> = details.map { it?.connection?.listItem() }
        .stateIn(viewModelScope, SharingStarted.Companion.Eagerly, null)

    val rows: StateFlow<List<GemListRow>> = details.map { it?.rows.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Companion.Eagerly, emptyList())

    fun disconnect(onSuccess: () -> Unit) {
        details.value?.connection?.connection?.session?.id?.let {
            viewModelScope.launch(ioDispatcher) {
                disconnectWalletConnection.disconnect(
                    connectionId = it,
                    onSuccess = { viewModelScope.launch(Dispatchers.Main) { onSuccess() } },
                    onError = { viewModelScope.launch(Dispatchers.Main) { onSuccess() } },
                )
            }
        } ?: onSuccess()
    }
}

private fun SavedStateHandle.requireString(argument: RouteArgument): String {
    val value = checkNotNull(get<String>(argument.key)) { "Missing route argument: ${argument.key}" }
    check(value.isNotBlank()) { "Blank route argument: ${argument.key}" }
    return value
}
