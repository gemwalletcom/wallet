package com.gemwallet.android.features.settings.networks.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.features.settings.networks.viewmodels.localization.string
import com.gemwallet.android.features.settings.networks.viewmodels.models.ServiceStatusRowUiModel
import com.gemwallet.android.features.settings.networks.viewmodels.models.ServiceStatusUIState
import com.gemwallet.android.features.settings.networks.viewmodels.models.tagStyle
import com.gemwallet.android.features.settings.networks.viewmodels.models.tagType
import com.gemwallet.android.features.settings.networks.viewmodels.models.uiModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.supervisorScope
import uniffi.gemstone.GemLatencyStatus
import uniffi.gemstone.GemServiceEndpoint
import uniffi.gemstone.GemServiceStatusInterface

@HiltViewModel
class ServiceStatusViewModel @Inject constructor(
    private val serviceStatus: GemServiceStatusInterface,
    @param:ApplicationContext private val context: Context,
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
                                        endpoint.toRow(statusState, context)
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
        return endpoints.map { it.toRow(GemLatencyStatus.Loading, context) }
    }
}

private fun GemServiceEndpoint.toRow(statusState: GemLatencyStatus, context: Context): ServiceStatusRowUiModel {
    val latency = statusState.uiModel(context)
    return ServiceStatusRowUiModel(
        id = url,
        model = ListItemModel(
            title = title(name = endpointType.name),
            titleTag = latency.text,
            titleTagStyle = latency.tagStyle(),
            titleTagType = latency.tagType(),
            titleExtra = host,
            titleExtraStyle = ListItemTextStyle.Body,
        ),
    )
}
