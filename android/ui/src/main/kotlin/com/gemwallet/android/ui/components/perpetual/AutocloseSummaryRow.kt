package com.gemwallet.android.ui.components.perpetual

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.space2

@Composable
fun AutocloseSummaryRow(
    takeProfitText: String?,
    stopLossText: String?,
    listPosition: ListPosition = ListPosition.Single,
) {
    val lines = listOfNotNull(
        takeProfitText?.let { "${stringResource(R.string.charts_take_profit)}: $it" },
        stopLossText?.let { "${stringResource(R.string.charts_stop_loss)}: $it" },
    )
    if (lines.isEmpty()) return
    PropertyItem(
        title = { PropertyTitleText(stringResource(R.string.perpetual_auto_close)) },
        data = {
            Column(
                horizontalAlignment = Alignment.End,
                verticalArrangement = Arrangement.spacedBy(space2),
            ) {
                lines.forEach { ListItemSupportText(it) }
            }
        },
        listPosition = listPosition,
    )
}
