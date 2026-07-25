package com.gemwallet.android.features.settings.networks.viewmodels.models

import com.wallet.core.primitives.ServiceStatusState
import uniffi.gemstone.GemServiceEndpointType

data class ServiceStatusUIState(
    val rows: List<ServiceStatusRowUiModel> = emptyList(),
)

data class ServiceStatusRowUiModel(
    val id: String,
    val type: GemServiceEndpointType,
    val flag: String,
    val host: String,
    val statusState: ServiceStatusState = ServiceStatusState.Loading,
)
