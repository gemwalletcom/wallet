package com.gemwallet.android.ui.components.chart

import android.content.Context
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.style.textStyle
import uniffi.gemstone.GemCandleTooltip
import uniffi.gemstone.GemCandleTooltipCell

data class CandlestickTooltipCellUIModel(val label: String, val value: String, val style: ListItemTextStyle)

data class CandlestickTooltipUIModel(val prices: List<CandlestickTooltipCellUIModel>, val summary: List<CandlestickTooltipCellUIModel>)

fun GemCandleTooltip.uiModel(context: Context): CandlestickTooltipUIModel = CandlestickTooltipUIModel(
    prices = prices.map { it.uiModel(context) },
    summary = summary.map { it.uiModel(context) },
)

private fun GemCandleTooltipCell.uiModel(context: Context): CandlestickTooltipCellUIModel = CandlestickTooltipCellUIModel(
    label = context.getString(row.stringRes()),
    value = value.text(),
    style = value.tone.textStyle(),
)
