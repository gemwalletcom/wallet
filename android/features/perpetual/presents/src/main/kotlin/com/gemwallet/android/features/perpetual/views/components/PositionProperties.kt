package com.gemwallet.android.features.perpetual.views.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDetailsDataAggregate
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingMiddle
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualMarginType

internal fun LazyListScope.positionProperties(
    position: PerpetualPositionDetailsDataAggregate?,
    onAutocloseClick: () -> Unit,
) {
    if (position == null) {
        return
    }
    item {
        SubheaderItem(R.string.perpetual_position)
    }
    item {
        PerpetualPositionItem(position, listPosition = ListPosition.First)
        PropertyItem(
            title = stringResource(R.string.perpetual_pnl),
            data = position.pnlWithPercentage,
            dataColor = position.pnlState.color(),
            listPosition = ListPosition.Middle,
        )
        AutocloseRow(position = position, onClick = onAutocloseClick)
        PropertyItem(
            title = stringResource(R.string.perpetual_size),
            data = position.size,
            listPosition = ListPosition.Middle,
        )
        PropertyItem(
            title = stringResource(R.string.perpetual_entry_price),
            data = position.entryPrice,
            listPosition = ListPosition.Middle,
        )
        if (position.liquidationPrice.isNotBlank()) {
            PropertyItem(
                title = stringResource(R.string.info_liquidation_price_title),
                data = position.liquidationPrice,
                info = InfoSheetEntity.LiquidationPriceInfo,
                listPosition = ListPosition.Middle,
            )
        }
        PropertyItem(
            title = stringResource(R.string.perpetual_margin),
            data = position.marginText(),
            listPosition = ListPosition.Middle,
        )
        PropertyItem(
            title = stringResource(R.string.info_funding_payments_title),
            data = position.fundingPayments,
            dataColor = position.fundingPaymentsDirection.color(),
            info = InfoSheetEntity.FundingPayments,
            listPosition = ListPosition.Last,
        )
    }
}

@Composable
private fun AutocloseRow(
    position: PerpetualPositionDetailsDataAggregate,
    onClick: () -> Unit,
) {
    val takeProfitText = position.takeProfit.formatTriggerOrder(stringResource(R.string.charts_take_profit))
    val stopLossText = position.stopLoss.formatTriggerOrder(stringResource(R.string.charts_stop_loss))
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
                    else -> ListItemSupportText("-")
                }
            }
            DataBadgeChevron()
        },
        listPosition = ListPosition.Middle,
    )
}

@Composable
private fun PerpetualPositionDetailsDataAggregate.marginText(): String {
    return "$marginAmount (${marginType.title()})"
}

@Composable
private fun PerpetualMarginType.title(): String {
    return when (this) {
        PerpetualMarginType.Cross -> stringResource(R.string.perpetual_margin_cross)
        PerpetualMarginType.Isolated -> stringResource(R.string.perpetual_margin_isolated)
    }
}

private fun Double?.formatTriggerOrder(label: String): String? {
    return this?.let { "$label: ${CurrencyFormatter(currency = Currency.USD).string(it)}" }
}
