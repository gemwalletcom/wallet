package com.gemwallet.android.ui.components.perpetual

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.perpetual.PerpetualConfirmDetailsUIModel
import com.gemwallet.android.ui.models.perpetual.PerpetualConfirmDetailsUIModel.Action

@Composable
fun PerpetualDetailsSummaryItem(
    model: PerpetualConfirmDetailsUIModel,
    onClick: () -> Unit,
    listPosition: ListPosition = ListPosition.Single,
) {
    PropertyItem(
        modifier = Modifier.clickable(onClick = onClick),
        title = { PropertyTitleText(R.string.common_details) },
        data = {
            PropertyDataText(
                text = model.summaryText().orEmpty(),
                color = model.summaryColor(),
                badge = { DataBadgeChevron() },
            )
        },
        listPosition = listPosition,
    )
}

@Composable
fun PerpetualDetailsBottomSheet(
    isVisible: Boolean,
    model: PerpetualConfirmDetailsUIModel?,
    onDismiss: () -> Unit,
) {
    if (model == null) return
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismiss,
        skipPartiallyExpanded = true,
        title = stringResource(R.string.common_details),
    ) {
        Column {
            PropertyItem(
                title = stringResource(R.string.perpetual_position),
                data = model.direction.titleAndLeverage(model.leverage),
                dataColor = model.direction.color(),
                listPosition = if (model.pnl != null) ListPosition.First else ListPosition.Single,
            )
            model.pnl?.let { pnl ->
                PropertyItem(
                    title = stringResource(R.string.perpetual_pnl),
                    data = pnl.text,
                    dataColor = pnl.direction.color(),
                    listPosition = ListPosition.Last,
                )
            }
            PropertyItem(
                title = stringResource(R.string.perpetual_margin),
                data = model.marginText,
                listPosition = ListPosition.First,
            )
            PropertyItem(
                title = stringResource(R.string.perpetual_size),
                data = model.sizeText,
                listPosition = ListPosition.Last,
            )
            model.autoclose?.let {
                AutocloseSummaryRow(
                    takeProfitText = it.takeProfitText,
                    stopLossText = it.stopLossText,
                )
            }
            PropertyItem(
                title = stringResource(R.string.perpetual_market_price),
                data = model.marketPriceText,
                listPosition = ListPosition.First,
            )
            model.entryPriceText?.let {
                PropertyItem(
                    title = stringResource(R.string.perpetual_entry_price),
                    data = it,
                    listPosition = ListPosition.Middle,
                )
            }
            PropertyItem(
                title = stringResource(R.string.swap_slippage),
                data = model.slippageText,
                listPosition = ListPosition.Last,
            )
        }
    }
}

@Composable
private fun PerpetualConfirmDetailsUIModel.summaryText(): String? = when (action) {
    Action.Open -> direction.titleAndLeverage(leverage)
    Action.Close -> pnl?.text
    Action.Increase -> stringResource(R.string.perpetual_increase_direction, direction.title())
    Action.Reduce -> stringResource(R.string.perpetual_reduce_direction, direction.title())
}

@Composable
private fun PerpetualConfirmDetailsUIModel.summaryColor(): Color = when (action) {
    Action.Open -> direction.color()
    Action.Close -> pnl?.direction?.color() ?: MaterialTheme.colorScheme.secondary
    Action.Increase, Action.Reduce -> MaterialTheme.colorScheme.secondary
}
