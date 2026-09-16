package com.gemwallet.android.features.settings.networks.viewmodels.models

data class ServiceStatusUIState(
    val rows: List<ServiceStatusRowUiModel> = emptyList(),
)

data class ServiceStatusRowUiModel(
    val id: String,
    val title: String,
    val host: String,
    val latency: LatencyUIModel,
)
