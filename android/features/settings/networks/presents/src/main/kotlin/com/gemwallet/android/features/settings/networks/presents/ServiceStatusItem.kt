package com.gemwallet.android.features.settings.networks.presents

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import com.gemwallet.android.features.settings.networks.viewmodels.models.ServiceStatusRowUiModel
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.features.settings.networks.presents.localization.string

@Composable
internal fun ServiceStatusItem(
    model: ServiceStatusRowUiModel,
    listPosition: ListPosition,
) {
    ListItem(
        title = {
            ListItemTitleText(
                text = model.endpoint.title(model.endpoint.endpointType.string()),
                titleBadge = {
                    LatencyStatusBadge(status = model.statusState)
                },
            )
        },
        subtitle = {
            ListItemSupportText(
                text = model.endpoint.host,
                color = MaterialTheme.colorScheme.onSurface,
            )
        },
        listPosition = listPosition,
    )
}
