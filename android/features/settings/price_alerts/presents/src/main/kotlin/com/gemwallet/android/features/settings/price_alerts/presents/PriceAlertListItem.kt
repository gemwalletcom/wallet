package com.gemwallet.android.features.settings.price_alerts.presents

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.price.tone
import com.gemwallet.android.features.settings.price_alerts.presents.localization.string
import com.gemwallet.android.features.settings.price_alerts.viewmodels.PriceAlertItemUIModel
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.PriceInfo
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.textStyle
import uniffi.gemstone.GemPriceAlertRow

internal fun priceAlertSupport(row: GemPriceAlertRow): (@Composable () -> Unit)? = {
    PriceInfo(
        price = row.prefix.string(),
        changes = row.suffix.string(),
        changeStyle = row.direction.tone().textStyle(),
        style = MaterialTheme.typography.bodyMedium,
    )
}

@Composable
internal fun PriceAlertAutoAssetItem(asset: AssetInfoDataAggregate, row: GemPriceAlertRow, enabled: Boolean, onCheckedChange: (Boolean) -> Unit) {
    AssetListItem(
        asset = asset,
        listPosition = ListPosition.Single,
        support = priceAlertSupport(row),
        badge = row.symbol,
        trailing = {
            Switch(
                checked = enabled,
                onCheckedChange = onCheckedChange,
            )
        },
    )
}

@Composable
internal fun PriceAlertAssetItem(item: PriceAlertItemUIModel, listPosition: ListPosition, modifier: Modifier = Modifier) {
    AssetListItem(
        modifier = modifier,
        asset = item.asset,
        listPosition = listPosition,
        icon = item.row.icon,
        support = priceAlertSupport(item.row),
        badge = item.row.symbol,
    )
}
