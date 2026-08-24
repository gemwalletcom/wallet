package com.gemwallet.android.ui.components.list_item.property

import androidx.annotation.StringRes
import androidx.compose.runtime.Composable
import com.gemwallet.android.ui.models.ListPosition

@Composable
fun MerchantPropertyItem(
    @StringRes title: Int,
    name: String,
    iconUrl: String? = null,
    listPosition: ListPosition = ListPosition.Middle,
) {
    PropertyItem(
        title = { PropertyTitleText(title) },
        data = {
            PropertyDataText(
                text = name,
                badge = iconUrl?.let { { DataBadgeChevron(icon = it, isShowChevron = false) } },
            )
        },
        listPosition = listPosition,
    )
}
