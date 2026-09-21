package com.gemwallet.android.features.perpetual.viewmodels.model

import android.content.Context
import com.gemwallet.android.features.perpetual.viewmodels.localization.stringRes
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemPerpetualButton
import uniffi.gemstone.GemPerpetualPositionDetail
import uniffi.gemstone.GemPerpetualPositionDetailRow
import uniffi.gemstone.GemPerpetualDetails
import uniffi.gemstone.GemPerpetualSection
import uniffi.gemstone.PerpetualPosition

data class PerpetualDetailsUIModel(
    val title: String,
    val sections: List<PerpetualDetailsSectionUIModel>,
    val modifyButtons: List<PerpetualButtonUIModel>,
    val position: PerpetualPosition?,
)

sealed interface PerpetualDetailsSectionUIModel {
    val title: String

    data class Position(override val title: String, val rows: List<PerpetualPositionRowUIModel>) : PerpetualDetailsSectionUIModel
    data class Info(override val title: String, val buttons: List<PerpetualButtonUIModel>, val rows: List<GemListRow>) : PerpetualDetailsSectionUIModel
}

sealed interface PerpetualPositionRowUIModel {
    val row: GemListRow

    data class Item(override val row: GemListRow) : PerpetualPositionRowUIModel
    data class Autoclose(override val row: GemListRow) : PerpetualPositionRowUIModel
}

data class PerpetualButtonUIModel(val title: String, val action: PerpetualButtonAction, val tone: PerpetualButtonTone)

enum class PerpetualButtonAction { OpenLong, OpenShort, Modify, Close, Increase, Reduce }

enum class PerpetualButtonTone { Positive, Negative, Primary }

internal fun GemPerpetualDetails.uiModel(context: Context): PerpetualDetailsUIModel = PerpetualDetailsUIModel(
    title = title,
    sections = sections.map { it.uiModel(context) },
    modifyButtons = modifyButtons.map { it.uiModel(context) },
    position = position,
)

internal fun GemPerpetualSection.uiModel(context: Context): PerpetualDetailsSectionUIModel = when (this) {
    is GemPerpetualSection.Position -> PerpetualDetailsSectionUIModel.Position(context.getString(stringRes()), rows.map { it.uiModel() })
    is GemPerpetualSection.Info -> PerpetualDetailsSectionUIModel.Info(context.getString(stringRes()), buttons.map { it.uiModel(context) }, rows)
}

internal fun GemPerpetualButton.uiModel(context: Context): PerpetualButtonUIModel = when (this) {
    GemPerpetualButton.LONG -> PerpetualButtonUIModel(context.getString(stringRes()), PerpetualButtonAction.OpenLong, PerpetualButtonTone.Positive)
    GemPerpetualButton.SHORT -> PerpetualButtonUIModel(context.getString(stringRes()), PerpetualButtonAction.OpenShort, PerpetualButtonTone.Negative)
    GemPerpetualButton.MODIFY -> PerpetualButtonUIModel(context.getString(stringRes()), PerpetualButtonAction.Modify, PerpetualButtonTone.Primary)
    GemPerpetualButton.CLOSE -> PerpetualButtonUIModel(context.getString(stringRes()), PerpetualButtonAction.Close, PerpetualButtonTone.Negative)
    GemPerpetualButton.INCREASE -> PerpetualButtonUIModel(context.getString(stringRes()), PerpetualButtonAction.Increase, PerpetualButtonTone.Primary)
    GemPerpetualButton.REDUCE -> PerpetualButtonUIModel(context.getString(stringRes()), PerpetualButtonAction.Reduce, PerpetualButtonTone.Negative)
}

internal fun GemPerpetualPositionDetail.uiModel(): PerpetualPositionRowUIModel = when (kind) {
    GemPerpetualPositionDetailRow.AUTOCLOSE -> PerpetualPositionRowUIModel.Autoclose(row)

    GemPerpetualPositionDetailRow.PNL,
    GemPerpetualPositionDetailRow.SIZE,
    GemPerpetualPositionDetailRow.ENTRY_PRICE,
    GemPerpetualPositionDetailRow.LIQUIDATION_PRICE,
    GemPerpetualPositionDetailRow.MARGIN,
    GemPerpetualPositionDetailRow.FUNDING_PAYMENTS,
    -> PerpetualPositionRowUIModel.Item(row)
}
