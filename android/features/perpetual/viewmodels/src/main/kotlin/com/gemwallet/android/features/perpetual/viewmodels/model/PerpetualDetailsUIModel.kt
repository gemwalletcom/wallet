package com.gemwallet.android.features.perpetual.viewmodels.model

import android.content.Context
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDetailsDataAggregate
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDetailsDataAggregate
import com.gemwallet.android.features.perpetual.viewmodels.localization.stringRes
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.style.textStyle
import com.gemwallet.android.ui.theme.Placeholder
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemPerpetual
import uniffi.gemstone.GemPerpetualButton
import uniffi.gemstone.GemPerpetualInfoRow
import uniffi.gemstone.GemPerpetualPositionDetailRow

sealed interface PerpetualDetailsSectionUIModel {
    val title: String

    data class Position(override val title: String, val rows: List<PerpetualPositionRowUIModel>) : PerpetualDetailsSectionUIModel
    data class Info(override val title: String, val buttons: List<PerpetualButtonUIModel>, val rows: List<ListItemModel>) : PerpetualDetailsSectionUIModel
}

sealed interface PerpetualPositionRowUIModel {
    data class Item(val model: ListItemModel) : PerpetualPositionRowUIModel
    data class Autoclose(val model: ListItemModel) : PerpetualPositionRowUIModel
}

data class PerpetualButtonUIModel(
    val title: String,
    val action: PerpetualButtonAction,
    val tone: PerpetualButtonTone,
)

enum class PerpetualButtonAction { OpenLong, OpenShort, Modify, Close, Increase, Reduce }

enum class PerpetualButtonTone { Positive, Negative, Primary }

internal fun GemPerpetualButton.uiModel(context: Context): PerpetualButtonUIModel = when (this) {
    GemPerpetualButton.LONG -> PerpetualButtonUIModel(context.getString(stringRes()), PerpetualButtonAction.OpenLong, PerpetualButtonTone.Positive)
    GemPerpetualButton.SHORT -> PerpetualButtonUIModel(context.getString(stringRes()), PerpetualButtonAction.OpenShort, PerpetualButtonTone.Negative)
    GemPerpetualButton.MODIFY -> PerpetualButtonUIModel(context.getString(stringRes()), PerpetualButtonAction.Modify, PerpetualButtonTone.Primary)
    GemPerpetualButton.CLOSE -> PerpetualButtonUIModel(context.getString(stringRes()), PerpetualButtonAction.Close, PerpetualButtonTone.Negative)
    GemPerpetualButton.INCREASE -> PerpetualButtonUIModel(context.getString(stringRes()), PerpetualButtonAction.Increase, PerpetualButtonTone.Primary)
    GemPerpetualButton.REDUCE -> PerpetualButtonUIModel(context.getString(stringRes()), PerpetualButtonAction.Reduce, PerpetualButtonTone.Negative)
}

internal fun PerpetualDetailsDataAggregate.infoListItem(context: Context, row: GemPerpetualInfoRow): ListItemModel = when (row) {
    GemPerpetualInfoRow.DAILY_VOLUME -> ListItemModel(title = context.getString(row.stringRes()), subtitle = dayVolume)
    GemPerpetualInfoRow.OPEN_INTEREST -> ListItemModel(title = context.getString(row.stringRes()), subtitle = openInterest, info = InfoSheetEntity.OpenInterestInfo)
    GemPerpetualInfoRow.FUNDING_RATE -> ListItemModel(title = context.getString(row.stringRes()), subtitle = funding, info = InfoSheetEntity.FundingAprInfo)
}

private val usdFormatter = CurrencyFormatter(currency = Currency.USD)

internal fun PerpetualPositionDetailsDataAggregate.positionRow(context: Context, row: GemPerpetualPositionDetailRow, perpetual: GemPerpetual): PerpetualPositionRowUIModel {
    val title = context.getString(row.stringRes())
    return when (row) {
        GemPerpetualPositionDetailRow.PNL -> PerpetualPositionRowUIModel.Item(ListItemModel(title = title, subtitle = pnlWithPercentage, subtitleStyle = pnlState.textStyle()))
        GemPerpetualPositionDetailRow.AUTOCLOSE -> {
            val takeProfitText = takeProfit?.let { perpetual.triggerOrderText(context.getString(R.string.perpetual_take_profit), usdFormatter.string(it)) }
            val stopLossText = stopLoss?.let { perpetual.triggerOrderText(context.getString(R.string.perpetual_stop_loss), usdFormatter.string(it)) }
            PerpetualPositionRowUIModel.Autoclose(
                ListItemModel(
                    title = title,
                    subtitle = takeProfitText ?: stopLossText ?: Placeholder.empty,
                    subtitleExtra = stopLossText.takeIf { takeProfitText != null },
                    info = InfoSheetEntity.AutoCloseInfo,
                ),
            )
        }
        GemPerpetualPositionDetailRow.SIZE -> PerpetualPositionRowUIModel.Item(ListItemModel(title = title, subtitle = size))
        GemPerpetualPositionDetailRow.ENTRY_PRICE -> PerpetualPositionRowUIModel.Item(ListItemModel(title = title, subtitle = entryPrice))
        GemPerpetualPositionDetailRow.LIQUIDATION_PRICE -> PerpetualPositionRowUIModel.Item(ListItemModel(title = title, subtitle = liquidationPrice, info = InfoSheetEntity.LiquidationPriceInfo))
        GemPerpetualPositionDetailRow.MARGIN -> PerpetualPositionRowUIModel.Item(ListItemModel(title = title, subtitle = perpetual.marginText(marginAmount, context.getString(marginType.stringRes()))))
        GemPerpetualPositionDetailRow.FUNDING_PAYMENTS -> PerpetualPositionRowUIModel.Item(
            ListItemModel(title = title, subtitle = fundingPayments, subtitleStyle = fundingPaymentsDirection.textStyle(), info = InfoSheetEntity.FundingPayments),
        )
    }
}
