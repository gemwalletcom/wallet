package com.gemwallet.android.features.asset.presents.details.components

import androidx.annotation.StringRes
import androidx.compose.foundation.clickable
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.models.ListPosition

@Composable
internal fun BalancePropertyItem(
    @StringRes title: Int,
    balance: String,
    listPosition: ListPosition,
    info: InfoSheetEntity? = null,
    onAction: (() -> Unit)?,
) {
    PropertyItem(
        modifier = if (onAction == null) Modifier else Modifier
            .clickable(onClick = onAction)
            .testTag("assetStake"),
        title = { PropertyTitleText(title, info = info) },
        data = { PropertyDataText(balance, badge = onAction?.let { { DataBadgeChevron() } }) },
        listPosition = listPosition,
    )
}