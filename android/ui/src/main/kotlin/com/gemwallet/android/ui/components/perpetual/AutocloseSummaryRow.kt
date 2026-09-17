package com.gemwallet.android.ui.components.perpetual

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.models.ListPosition

@Composable
fun AutocloseSummaryRow(
    takeProfitText: String?,
    stopLossText: String?,
    listPosition: ListPosition = ListPosition.Single,
) {
    val lines = autocloseSummaryLines(LocalContext.current, takeProfitText, stopLossText)
    if (lines.isEmpty()) return
    ListItem(
        model = ListItemModel(
            title = stringResource(R.string.perpetual_auto_close),
            subtitle = lines.first(),
            subtitleExtra = lines.getOrNull(1),
        ),
        listPosition = listPosition,
    )
}
