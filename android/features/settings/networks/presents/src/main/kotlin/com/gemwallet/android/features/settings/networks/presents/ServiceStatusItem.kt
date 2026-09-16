package com.gemwallet.android.features.settings.networks.presents

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import com.gemwallet.android.features.settings.networks.viewmodels.models.ServiceStatusRowUiModel
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
                text = model.title,
                titleBadge = {
                    LatencyStatusBadge(latency = model.latency)
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
