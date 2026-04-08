package com.gemwallet.android.features.settings.price_alerts.presents

import androidx.annotation.StringRes
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.pricealerts.aggregates.PriceAlertDataAggregate
import com.gemwallet.android.domains.pricealerts.aggregates.PriceAlertType
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.AssetInfoUIModel
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.PriceInfo
import com.gemwallet.android.ui.components.list_item.assetPriceSupport
import com.gemwallet.android.ui.models.ListPosition

internal data class PriceAlertSupportContent(
    @param:StringRes val labelRes: Int? = null,
    val primaryText: String = "",
    val secondaryText: String = "",
) {
    val hasContent: Boolean
        get() = labelRes != null || primaryText.isNotEmpty() || secondaryText.isNotEmpty()
}

internal fun priceAlertSupportContent(
    type: PriceAlertType,
    price: String,
    percentage: String,
): PriceAlertSupportContent = when (type) {
    PriceAlertType.Auto -> PriceAlertSupportContent(
        primaryText = price,
        secondaryText = percentage,
    )
    PriceAlertType.Over -> PriceAlertSupportContent(
        labelRes = R.string.price_alerts_direction_over,
        secondaryText = price,
    )
    PriceAlertType.Under -> PriceAlertSupportContent(
        labelRes = R.string.price_alerts_direction_under,
        secondaryText = price,
    )
    PriceAlertType.Increase -> PriceAlertSupportContent(
        labelRes = R.string.price_alerts_direction_increases_by,
        secondaryText = percentage,
    )
    PriceAlertType.Decrease -> PriceAlertSupportContent(
        labelRes = R.string.price_alerts_direction_decreases_by,
        secondaryText = percentage,
    )
}

internal fun priceAlertSupport(item: PriceAlertDataAggregate): (@Composable () -> Unit)? {
    val content = priceAlertSupportContent(
        type = item.type,
        price = item.price,
        percentage = item.percentage,
    )
    if (!content.hasContent) {
        return null
    }
    return {
        val primaryText = content.labelRes?.let { stringResource(it) } ?: content.primaryText
        PriceInfo(
            price = primaryText,
            changes = content.secondaryText,
            state = item.priceState,
            style = MaterialTheme.typography.bodyMedium,
        )
    }
}

@Composable
internal fun PriceAlertAutoAssetItem(
    assetInfo: AssetInfo,
    enabled: Boolean,
    onCheckedChange: (Boolean) -> Unit,
) {
    val uiModel = remember(assetInfo) { AssetInfoUIModel(assetInfo) }
    AssetListItem(
        asset = uiModel,
        listPosition = ListPosition.Single,
        support = assetPriceSupport(uiModel.price),
        badge = uiModel.symbol.uppercase(),
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
