package com.gemwallet.android.features.bridge.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.ui.models.navigation.RouteArgument
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.emitAll
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class ConnectionViewModel @Inject constructor(
    private val bridgesRepository: BridgesRepository,
    savedState: SavedStateHandle
) : ViewModel() {

    private val connectionId = savedState.requireString(RouteArgument.ConnectionId)

    val connection = flow { emitAll(bridgesRepository.getConnections(connectionId)) }
        .stateIn(viewModelScope, SharingStarted.Companion.Eagerly, null)

    fun disconnect(onSuccess: () -> Unit) {
        connection.value?.session?.id?.let {
            viewModelScope.launch(Dispatchers.IO) {
                bridgesRepository.disconnect(
                    id = it,
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
