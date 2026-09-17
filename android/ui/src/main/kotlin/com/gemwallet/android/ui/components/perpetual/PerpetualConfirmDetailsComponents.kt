package com.gemwallet.android.ui.components.perpetual

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.perpetual.PerpetualConfirmDetailsUIModel
import com.gemwallet.android.ui.style.textStyle
import uniffi.gemstone.GemPerpetualDetailsAction.CLOSE
import uniffi.gemstone.GemPerpetualDetailsAction.INCREASE
import uniffi.gemstone.GemPerpetualDetailsAction.OPEN
import uniffi.gemstone.GemPerpetualDetailsAction.REDUCE

@Composable
fun PerpetualDetailsSummaryItem(
    model: PerpetualConfirmDetailsUIModel,
    onClick: () -> Unit,
    listPosition: ListPosition = ListPosition.Single,
) {
    ListItem(
        model = ListItemModel(
            title = stringResource(R.string.common_details),
            subtitle = model.summaryText().orEmpty(),
            subtitleStyle = model.summaryStyle(),
        ),
        listPosition = listPosition,
        modifier = Modifier.clickable(onClick = onClick),
        accessory = { DataBadgeChevron() },
    )
}

@Composable
fun PerpetualDetailsBottomSheet(
    isVisible: Boolean,
    model: PerpetualConfirmDetailsUIModel?,
    onDismiss: () -> Unit,
) {
    ModalBottomSheet(
        item = model.takeIf { isVisible },
        onDismissRequest = onDismiss,
        expansion = SheetExpansion.Full,
        title = { stringResource(R.string.common_details) },
    ) { model ->
        Column {
            ListItem(
                model = ListItemModel(
                    title = stringResource(R.string.perpetual_position),
                    subtitle = model.direction.titleAndLeverage(model.leverage),
                    subtitleStyle = model.direction.textStyle(),
                ),
                listPosition = if (model.pnl != null) ListPosition.First else ListPosition.Single,
            )
            model.pnl?.let { pnl ->
                ListItem(
                    model = ListItemModel(title = stringResource(R.string.perpetual_pnl), subtitle = pnl.text, subtitleStyle = pnl.direction.textStyle()),
                    listPosition = ListPosition.Last,
                )
            }
            ListItem(model = ListItemModel(title = stringResource(R.string.perpetual_margin), subtitle = model.marginText), listPosition = ListPosition.First)
            ListItem(model = ListItemModel(title = stringResource(R.string.perpetual_size), subtitle = model.sizeText), listPosition = ListPosition.Last)
            model.autoclose?.let {
                AutocloseSummaryRow(
                    takeProfitText = it.takeProfitText,
                    stopLossText = it.stopLossText,
                )
            }
            ListItem(model = ListItemModel(title = stringResource(R.string.perpetual_market_price), subtitle = model.marketPriceText), listPosition = ListPosition.First)
            model.entryPriceText?.let {
                ListItem(model = ListItemModel(title = stringResource(R.string.perpetual_entry_price), subtitle = it), listPosition = ListPosition.Middle)
            }
            ListItem(model = ListItemModel(title = stringResource(R.string.swap_slippage), subtitle = model.slippageText), listPosition = ListPosition.Last)
        }
    }
}

@Composable
private fun PerpetualConfirmDetailsUIModel.summaryText(): String? = when (action) {
    OPEN -> direction.titleAndLeverage(leverage)
    CLOSE -> pnl?.text
    INCREASE -> stringResource(R.string.perpetual_increase_direction, direction.title())
    REDUCE -> stringResource(R.string.perpetual_reduce_direction, direction.title())
}

private fun PerpetualConfirmDetailsUIModel.summaryStyle(): ListItemTextStyle = when (action) {
    OPEN -> direction.textStyle()
    CLOSE -> pnl?.direction?.textStyle() ?: ListItemTextStyle.Secondary
    INCREASE, REDUCE -> ListItemTextStyle.Secondary
}
