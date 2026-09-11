package com.gemwallet.android.features.settings.networks.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.features.settings.networks.viewmodels.models.ServiceStatusRowUiModel
import com.gemwallet.android.features.settings.networks.viewmodels.models.ServiceStatusUIState
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.supervisorScope
import uniffi.gemstone.GemLatencyStatus
import uniffi.gemstone.GemServiceEndpoint
import uniffi.gemstone.GemServiceStatus
import javax.inject.Inject

@HiltViewModel
class ServiceStatusViewModel @Inject constructor(
    private val serviceStatus: GemServiceStatus,
) : ViewModel() {
    private val endpoints = serviceStatus.getEndpoints()

    private val _uiState = MutableStateFlow(ServiceStatusUIState(rows = loadingRows()))
    val uiState = _uiState.asStateFlow()

    private var fetchJob: Job? = null

    fun fetch() {
        fetchJob?.cancel()
        fetchJob = viewModelScope.launch {
            _uiState.value = ServiceStatusUIState(rows = loadingRows())

            supervisorScope {
                endpoints.forEach { endpoint ->
                    launch {
                        val statusState = serviceStatus.getEndpointStatus(endpoint.url)
                        _uiState.update { current ->
                            current.copy(
                                rows = current.rows.map {
                                    if (it.id == endpoint.url) {
                                        endpoint.toRow(statusState)
                                    } else {
                                        it
                                    }
                                },
                            )
                        }
                    }
                }
            }

        }
    }

    private fun loadingRows(): List<ServiceStatusRowUiModel> {
        return endpoints.map { it.toRow(GemLatencyStatus.Loading) }
    }
}

private fun GemServiceEndpoint.toRow(statusState: GemLatencyStatus): ServiceStatusRowUiModel {
    return ServiceStatusRowUiModel(
        id = url,
        type = endpointType,
        flag = flag,
        host = host,
        statusState = statusState,
    )
}
