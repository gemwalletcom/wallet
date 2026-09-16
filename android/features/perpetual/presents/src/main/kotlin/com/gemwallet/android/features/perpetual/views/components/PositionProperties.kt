package com.gemwallet.android.features.perpetual.views.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDetailsDataAggregate
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingMiddle
import com.wallet.core.primitives.Currency
import com.gemwallet.android.features.perpetual.localization.stringRes
import com.wallet.core.primitives.PerpetualMarginType
import com.gemwallet.android.ui.theme.Placeholder
import uniffi.gemstone.PerpetualProvider
import uniffi.gemstone.GemPerpetual
import uniffi.gemstone.GemPerpetualPositionDetailRow

private val usdFormatter = CurrencyFormatter(currency = Currency.USD)

internal fun LazyListScope.positionProperties(
    position: PerpetualPositionDetailsDataAggregate?,
    rows: List<GemPerpetualPositionDetailRow>,
    onAutocloseClick: () -> Unit,
) {
    if (position == null) {
        return
    }
    item {
        PerpetualPositionItem(position, listPosition = ListPosition.First)
    }
    itemsIndexed(rows) { index, row ->
        val listPosition = if (index == rows.lastIndex) ListPosition.Last else ListPosition.Middle
        when (row) {
            GemPerpetualPositionDetailRow.PNL -> PropertyItem(
                title = stringResource(row.stringRes()),
                data = position.pnlWithPercentage,
                dataColor = position.pnlState.color(),
                listPosition = listPosition,
            )
            GemPerpetualPositionDetailRow.AUTOCLOSE -> AutocloseRow(position = position, listPosition = listPosition, onClick = onAutocloseClick)
            GemPerpetualPositionDetailRow.SIZE -> PropertyItem(
                title = stringResource(row.stringRes()),
                data = position.size,
                listPosition = listPosition,
            )
            GemPerpetualPositionDetailRow.ENTRY_PRICE -> PropertyItem(
                title = stringResource(row.stringRes()),
                data = position.entryPrice,
                listPosition = listPosition,
            )
            GemPerpetualPositionDetailRow.LIQUIDATION_PRICE -> PropertyItem(
                title = stringResource(row.stringRes()),
                data = position.liquidationPrice,
                info = InfoSheetEntity.LiquidationPriceInfo,
                listPosition = listPosition,
            )
            GemPerpetualPositionDetailRow.MARGIN -> PropertyItem(
                title = stringResource(row.stringRes()),
                data = position.marginText(),
                listPosition = listPosition,
            )
            GemPerpetualPositionDetailRow.FUNDING_PAYMENTS -> PropertyItem(
                title = stringResource(row.stringRes()),
                data = position.fundingPayments,
                dataColor = position.fundingPaymentsDirection.color(),
                info = InfoSheetEntity.FundingPayments,
                listPosition = listPosition,
            )
        }
    }
}

@Composable
private fun AutocloseRow(
    position: PerpetualPositionDetailsDataAggregate,
    listPosition: ListPosition,
    onClick: () -> Unit,
) {
    val takeProfitText = position.takeProfit.formatTriggerOrder(stringResource(R.string.perpetual_take_profit))
    val stopLossText = position.stopLoss.formatTriggerOrder(stringResource(R.string.perpetual_stop_loss))
    PropertyItem(
        modifier = Modifier.clickable(onClick = onClick),
        title = {
            PropertyTitleText(
                text = stringResource(R.string.perpetual_auto_close),
                info = InfoSheetEntity.AutoCloseInfo,
            )
        },
        data = {
            Column(horizontalAlignment = Alignment.End) {
                when {
                    takeProfitText != null && stopLossText != null -> {
                        ListItemSupportText(takeProfitText)
                        ListItemSupportText(stopLossText)
                    }
                    takeProfitText != null -> ListItemSupportText(takeProfitText)
                    stopLossText != null -> ListItemSupportText(stopLossText)
                    else -> ListItemSupportText(Placeholder.empty)
                }
            }
            DataBadgeChevron()
        },
        listPosition = listPosition,
    )
}

private val perpetual = GemPerpetual(PerpetualProvider.HYPERCORE)

@Composable
private fun PerpetualPositionDetailsDataAggregate.marginText(): String {
    return perpetual.marginText(marginAmount, stringResource(marginType.stringRes()))
}

private fun Double?.formatTriggerOrder(label: String): String? {
    return this?.let { perpetual.triggerOrderText(label, usdFormatter.string(it)) }
}
