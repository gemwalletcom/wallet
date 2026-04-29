package com.gemwallet.android.features.settings.networks.presents

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.settings.networks.viewmodels.models.ServiceStatusEndpointType
import com.gemwallet.android.features.settings.networks.viewmodels.models.ServiceStatusRowUiModel
import com.gemwallet.android.features.settings.networks.viewmodels.models.ServiceStatusState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.models.ListPosition

@Composable
internal fun ServiceStatusItem(
    model: ServiceStatusRowUiModel,
    listPosition: ListPosition,
) {
    ListItem(
        title = {
            ListItemTitleText(
                text = model.title(),
                titleBadge = {
                    LatencyStatusBadge(
                        latency = model.statusState.latency,
                        isLoading = model.statusState is ServiceStatusState.Loading,
                    )
                },
            )
        },
        subtitle = {
            ListItemSupportText(
                text = model.host,
                color = MaterialTheme.colorScheme.onSurface,
            )
        },
        listPosition = listPosition,
    )
}

@Composable
private fun ServiceStatusRowUiModel.title(): String {
    val name = when (type) {
        ServiceStatusEndpointType.Api -> "API"
        ServiceStatusEndpointType.GemNode -> stringResource(R.string.nodes_gem_wallet_node)
    }
    return flag?.let { "$name $it" } ?: name
}

private val ServiceStatusState.latency: ULong?
    get() = (this as? ServiceStatusState.Result)?.latency
