package com.gemwallet.android.ui.components.list_item.property

import androidx.annotation.StringRes
import androidx.compose.runtime.Composable
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.model.PriceChangeFormatter
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.models.ListPosition

@Composable
fun PnlPropertyItem(
    @StringRes title: Int,
    value: Double,
    formatter: PriceChangeFormatter,
    listPosition: ListPosition,
) {
    PropertyItem(
        title = title,
        data = formatter.string(value),
        dataColor = value.toValueDirection().color(),
        listPosition = listPosition,
    )
}
