package com.gemwallet.android.features.settings.price_alerts.presents

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.price.tone
import com.gemwallet.android.domains.pricealerts.aggregates.PriceAlertDataAggregate
import com.gemwallet.android.features.settings.price_alerts.presents.localization.string
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.PriceInfo
import com.gemwallet.android.ui.components.list_item.assetPriceSupport
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.textStyle

internal fun priceAlertSupport(item: PriceAlertDataAggregate): (@Composable () -> Unit)? = {
    PriceInfo(
        price = item.prefix.string(),
        changes = item.suffix.string(),
        changeStyle = item.priceDirection.tone().textStyle(),
        style = MaterialTheme.typography.bodyMedium,
    )
}

@Composable
internal fun PriceAlertAutoAssetItem(asset: AssetInfoDataAggregate, enabled: Boolean, onCheckedChange: (Boolean) -> Unit) {
    AssetListItem(
        asset = asset,
        listPosition = ListPosition.Single,
        support = assetPriceSupport(asset.price),
        badge = asset.asset.symbol,
        trailing = {
            Switch(
                checked = enabled,
                onCheckedChange = onCheckedChange,
            )
        },
    )
}

@Composable
internal fun PriceAlertAssetItem(item: PriceAlertDataAggregate, listPosition: ListPosition, modifier: Modifier = Modifier) {
    AssetListItem(
        modifier = modifier,
        asset = item.asset,
        listPosition = listPosition,
        support = priceAlertSupport(item),
        badge = item.titleBadge,
    )
}
