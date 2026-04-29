package com.gemwallet.android.features.settings.networks.viewmodels.models

data class ServiceStatusUIState(
    val rows: List<ServiceStatusRowUiModel> = emptyList(),
    val isRefreshing: Boolean = false,
)

data class ServiceStatusRowUiModel(
    val id: String,
    val type: ServiceStatusEndpointType,
    val flag: String?,
    val host: String,
    val statusState: ServiceStatusState = ServiceStatusState.Loading,
)

enum class ServiceStatusEndpointType {
    Api,
    GemNode,
}

sealed interface ServiceStatusState {
    data object Loading : ServiceStatusState

    data object Error : ServiceStatusState

    data class Result(
        val latency: ULong,
    ) : ServiceStatusState
}
