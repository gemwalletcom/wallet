package com.gemwallet.android.features.wallet_connector.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.wallet_connect.cases.DisconnectWalletConnection
import com.gemwallet.android.data.services.store.queries.ConnectionQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.navigation.RouteArgument
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemConnectionRow
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemWalletConnectServiceInterface
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ConnectionViewModel @Inject constructor(
    connectionQuery: ConnectionQuery,
    private val disconnectWalletConnection: DisconnectWalletConnection,
    private val service: GemWalletConnectServiceInterface,
    savedState: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val connectionId = savedState.requireString(RouteArgument.ConnectionId)

    val details = connectionQuery(connectionId)
        .mapLatest { connection -> connection?.let { service.connectionDetails(it.toGem()) } }
        .stateIn(viewModelScope, SharingStarted.Companion.Eagerly, null)

    val connectionRow: StateFlow<GemConnectionRow?> = details.map { it?.connection?.row }
        .stateIn(viewModelScope, SharingStarted.Companion.Eagerly, null)

    val rows: StateFlow<List<GemListRow>> = details.map { it?.rows.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Companion.Eagerly, emptyList())

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

    fun disconnect(onSuccess: () -> Unit) {
        details.value?.connection?.connection?.session?.id?.let {
            viewModelScope.launch(ioDispatcher) {
                disconnectWalletConnection.disconnect(
                    connectionId = it,
                    onSuccess = { viewModelScope.launch(Dispatchers.Main) { onSuccess() } },
                    onError = { error -> errorState.value = error.text(context) },
                )
            }
        } ?: onSuccess()
    }

    fun clearError() = errorState.update { null }
}

private fun SavedStateHandle.requireString(argument: RouteArgument): String {
    val value = checkNotNull(get<String>(argument.key)) { "Missing route argument: ${argument.key}" }
    check(value.isNotBlank()) { "Blank route argument: ${argument.key}" }
    return value
}
