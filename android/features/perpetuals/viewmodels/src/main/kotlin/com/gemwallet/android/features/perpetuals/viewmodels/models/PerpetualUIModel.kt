package com.gemwallet.android.features.perpetuals.viewmodels.models

import android.content.Context
import com.gemwallet.android.ui.localization.stringRes
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemPerpetualButton
import uniffi.gemstone.GemPerpetualButtonRow
import uniffi.gemstone.GemPerpetualDetails
import uniffi.gemstone.GemPerpetualPositionDetail
import uniffi.gemstone.GemPerpetualPositionDetailRow
import uniffi.gemstone.GemPerpetualSection
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.PerpetualPosition

data class PerpetualUIModel(val title: String, val sections: List<PerpetualSectionUIModel>, val modifyButtons: List<PerpetualButtonUIModel>, val position: PerpetualPosition?)

sealed interface PerpetualSectionUIModel {
    val title: String

    data class Position(override val title: String, val rows: List<PerpetualPositionDetailUIModel>) : PerpetualSectionUIModel
    data class Info(override val title: String, val buttons: List<PerpetualButtonUIModel>, val rows: List<GemListRow>) : PerpetualSectionUIModel
}

sealed interface PerpetualPositionDetailUIModel {
    val row: GemListRow

    data class Item(override val row: GemListRow) : PerpetualPositionDetailUIModel
    data class Autoclose(override val row: GemListRow) : PerpetualPositionDetailUIModel
}

data class PerpetualButtonUIModel(val title: String, val action: GemPerpetualButton, val tone: GemValueTone)

internal fun GemPerpetualDetails.uiModel(context: Context): PerpetualUIModel = PerpetualUIModel(
    title = title,
    sections = sections.map { it.uiModel(context) },
    modifyButtons = modifyButtons.map { it.uiModel(context) },
    position = position,
)

internal fun GemPerpetualSection.uiModel(context: Context): PerpetualSectionUIModel = when (this) {
    is GemPerpetualSection.Position -> PerpetualSectionUIModel.Position(context.getString(stringRes()), rows.map { it.uiModel() })
    is GemPerpetualSection.Info -> PerpetualSectionUIModel.Info(context.getString(stringRes()), buttons.map { it.uiModel(context) }, rows)
}

internal fun GemPerpetualButtonRow.uiModel(context: Context): PerpetualButtonUIModel = PerpetualButtonUIModel(context.getString(button.stringRes()), button, tone)

internal fun GemPerpetualPositionDetail.uiModel(): PerpetualPositionDetailUIModel = when (kind) {
    GemPerpetualPositionDetailRow.AUTOCLOSE -> PerpetualPositionDetailUIModel.Autoclose(row)

    GemPerpetualPositionDetailRow.PNL,
    GemPerpetualPositionDetailRow.SIZE,
    GemPerpetualPositionDetailRow.ENTRY_PRICE,
    GemPerpetualPositionDetailRow.LIQUIDATION_PRICE,
    GemPerpetualPositionDetailRow.MARGIN,
    GemPerpetualPositionDetailRow.FUNDING_PAYMENTS,
    -> PerpetualPositionDetailUIModel.Item(row)
}
