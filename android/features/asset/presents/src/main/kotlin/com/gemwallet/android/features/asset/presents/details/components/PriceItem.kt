package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.color
import com.wallet.core.primitives.AssetId

internal fun LazyListScope.price(
    uiState: AssetInfoUIModel,
    onChart: (AssetId) -> Unit,
    onPriceAlerts: (AssetId) -> Unit
) {
    item {
        ListItem(
            model = uiState.priceListItem,
            listPosition = ListPosition.First,
            modifier = Modifier
                .clickable { onChart(uiState.asset.id) }
                .testTag("assetChart"),
            accessory = {
                DataBadgeChevron {
                    Text(
                        text = uiState.priceDayChanges,
                        color = uiState.priceChangedType.color(),
                        style = MaterialTheme.typography.bodyLarge,
                    )
                }
            },
        )
    }

    if (uiState.detailsState.showsPriceAlerts) {
        item {
            ListItem(
                model = uiState.priceAlertsListItem,
                listPosition = ListPosition.Middle,
                modifier = Modifier
                    .clickable { onPriceAlerts(uiState.asset.id) }
                    .testTag("assetChart"),
                accessory = { DataBadgeChevron() },
            )
        }
    }
}
