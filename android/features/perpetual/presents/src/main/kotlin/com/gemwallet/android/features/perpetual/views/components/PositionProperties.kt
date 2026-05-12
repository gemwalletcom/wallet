package com.gemwallet.android.features.perpetual.views.components

import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDetailsDataAggregate
import com.gemwallet.android.model.format
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualMarginType

internal fun LazyListScope.positionProperties(position: PerpetualPositionDetailsDataAggregate?) {
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
        PropertyItem(
            title = stringResource(R.string.perpetual_auto_close),
            data = position.autoCloseText(),
            info = InfoSheetEntity.AutoCloseInfo,
            listPosition = ListPosition.Middle,
        )
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
private fun PerpetualPositionDetailsDataAggregate.autoCloseText(): String {
    val takeProfit = takeProfit.formatTriggerOrder(stringResource(R.string.charts_take_profit))
    val stopLoss = stopLoss.formatTriggerOrder(stringResource(R.string.charts_stop_loss))
    return when {
        takeProfit != null && stopLoss != null -> "$takeProfit / $stopLoss"
        takeProfit != null -> takeProfit
        stopLoss != null -> stopLoss
        else -> "-"
    }
}

@Composable
private fun PerpetualPositionDetailsDataAggregate.marginText(): String {
    return "${marginAmount} (${marginType.title()})"
}

@Composable
private fun PerpetualMarginType.title(): String {
    return when (this) {
        PerpetualMarginType.Cross -> stringResource(R.string.perpetual_margin_cross)
        PerpetualMarginType.Isolated -> stringResource(R.string.perpetual_margin_isolated)
    }
}

private fun Double?.formatTriggerOrder(label: String): String? {
    return this?.let { "$label: ${Currency.USD.format(it, dynamicPlace = true)}" }
}
