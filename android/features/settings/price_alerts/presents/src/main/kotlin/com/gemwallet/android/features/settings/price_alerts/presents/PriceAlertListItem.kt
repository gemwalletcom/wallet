package com.gemwallet.android.features.settings.price_alerts.presents

import androidx.annotation.StringRes
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.pricealerts.aggregates.PriceAlertDataAggregate
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemAssetRowTitle
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregate
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.PriceInfo
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.features.settings.price_alerts.presents.localization.string
import com.gemwallet.android.ui.components.list_item.assetPriceSupport
import com.gemwallet.android.ui.models.ListPosition

internal fun priceAlertSupport(item: PriceAlertDataAggregate): (@Composable () -> Unit)? = {
    PriceInfo(
        price = item.prefix.string(),
        changes = item.suffix.string(),
        state = item.priceDirection.toValueDirection(),
        style = MaterialTheme.typography.bodyMedium,
    )
}

@Composable
internal fun PriceAlertAutoAssetItem(
    assetInfo: AssetInfo,
    enabled: Boolean,
    onCheckedChange: (Boolean) -> Unit,
) {
    val uiModel = remember(assetInfo) { assetInfo.toAssetInfoDataAggregate(GemAssetRowTitle.CANONICAL_ASSET) }
    AssetListItem(
        asset = uiModel,
        listPosition = ListPosition.Single,
        support = assetPriceSupport(uiModel.price),
        badge = uiModel.asset.symbol.uppercase(),
        trailing = {
            Switch(
                checked = enabled,
                onCheckedChange = onCheckedChange,
            )
        },
    )
}

@Composable
internal fun PriceAlertAssetItem(
    item: PriceAlertDataAggregate,
    listPosition: ListPosition,
    modifier: Modifier = Modifier,
) {
    AssetListItem(
        modifier = modifier,
        asset = item.asset,
        listPosition = listPosition,
        support = priceAlertSupport(item),
        badge = item.titleBadge,
    )
}
