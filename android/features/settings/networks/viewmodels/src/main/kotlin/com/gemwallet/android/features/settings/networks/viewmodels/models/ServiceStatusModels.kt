package com.gemwallet.android.features.settings.networks.viewmodels.models

import uniffi.gemstone.GemLatencyStatus
import uniffi.gemstone.GemServiceEndpoint

data class ServiceStatusUIState(
    val rows: List<ServiceStatusRowUiModel> = emptyList(),
)

data class ServiceStatusRowUiModel(
    val endpoint: GemServiceEndpoint,
    val statusState: GemLatencyStatus = GemLatencyStatus.Loading,
) {
    val id: String get() = endpoint.url
}
