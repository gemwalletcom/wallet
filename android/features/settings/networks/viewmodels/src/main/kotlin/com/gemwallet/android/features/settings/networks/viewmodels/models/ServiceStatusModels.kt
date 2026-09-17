package com.gemwallet.android.features.settings.networks.viewmodels.models

import com.gemwallet.android.ui.components.list_item.ListItemModel

data class ServiceStatusUIState(
    val rows: List<ServiceStatusRowUiModel> = emptyList(),
)

data class ServiceStatusRowUiModel(
    val id: String,
    val model: ListItemModel,
)
