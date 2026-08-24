package com.gemwallet.android.ui.components.list_item.property

import androidx.annotation.StringRes
import androidx.compose.runtime.Composable
import com.gemwallet.android.ui.models.ListPosition

@Composable
fun IconPropertyItem(
    @StringRes title: Int,
    text: String,
    icon: Any? = null,
    listPosition: ListPosition = ListPosition.Middle,
) {
    PropertyItem(
        title = { PropertyTitleText(title) },
        data = {
            PropertyDataText(
                text = text,
                badge = icon?.let { { DataBadgeChevron(icon = it, isShowChevron = false) } },
            )
        },
        listPosition = listPosition,
    )
}
