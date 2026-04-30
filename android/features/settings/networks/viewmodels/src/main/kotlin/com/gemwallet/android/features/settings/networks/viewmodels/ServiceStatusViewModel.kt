package com.gemwallet.android.features.settings.networks.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.Constants
import com.gemwallet.android.blockchain.services.NodeStatusService
import com.gemwallet.android.cases.nodes.GemNodeRegion
import com.gemwallet.android.features.settings.networks.viewmodels.models.ServiceStatusEndpointType
import com.gemwallet.android.features.settings.networks.viewmodels.models.ServiceStatusRowUiModel
import com.gemwallet.android.features.settings.networks.viewmodels.models.ServiceStatusState
import com.gemwallet.android.features.settings.networks.viewmodels.models.ServiceStatusUIState
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.supervisorScope
import java.net.URL
import javax.inject.Inject

@HiltViewModel
class ServiceStatusViewModel @Inject constructor(
    private val nodeStatusService: NodeStatusService,
) : ViewModel() {
    private val endpoints = listOf(
        ServiceStatusEndpoint(
            type = ServiceStatusEndpointType.Api,
            flag = GemNodeRegion.US.flag,
            url = Constants.API_URL,
            host = Constants.API_HOST,
        ),
    ) + GemNodeRegion.entries.map {
        ServiceStatusEndpoint(
            type = ServiceStatusEndpointType.GemNode,
            flag = it.flag,
            url = it.baseUrl,
            host = URL(it.baseUrl).host,
        )
    }

    private val _uiState = MutableStateFlow(ServiceStatusUIState(rows = loadingRows()))
    val uiState = _uiState.asStateFlow()

    private var fetchJob: Job? = null

    init {
        fetch()
    }

    fun fetch() {
        fetchJob?.cancel()
        fetchJob = viewModelScope.launch {
            _uiState.value = ServiceStatusUIState(
                rows = loadingRows(),
                isRefreshing = true,
            )

            supervisorScope {
                endpoints.forEach { endpoint ->
                    launch {
                        val statusState = status(endpoint)
                        _uiState.update { current ->
                            current.copy(
                                rows = current.rows.map {
                                    if (it.id == endpoint.id) {
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

            _uiState.update { it.copy(isRefreshing = false) }
        }
    }

    private fun loadingRows(): List<ServiceStatusRowUiModel> {
        return endpoints.map { it.toRow(ServiceStatusState.Loading) }
    }

    private suspend fun status(endpoint: ServiceStatusEndpoint): ServiceStatusState {
        return nodeStatusService.getEndpointLatency(endpoint.url)
            ?.let { ServiceStatusState.Result(it) }
            ?: ServiceStatusState.Error
    }
}

private data class ServiceStatusEndpoint(
    val type: ServiceStatusEndpointType,
    val flag: String?,
    val url: String,
    val host: String,
) {
    val id: String get() = url

    fun toRow(statusState: ServiceStatusState): ServiceStatusRowUiModel {
        return ServiceStatusRowUiModel(
            id = id,
            type = type,
            flag = flag,
            host = host,
            statusState = statusState,
        )
    }
}
