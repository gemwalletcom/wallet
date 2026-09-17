package com.gemwallet.android.features.settings.settings.viewmodels.models

import android.content.Context
import com.gemwallet.android.features.settings.settings.viewmodels.localization.stringRes
import com.gemwallet.android.features.settings.settings.viewmodels.style.icon
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.models.actions.PreferencesAction
import com.wallet.core.primitives.Appearance
import uniffi.gemstone.GemPreferencesRow
import uniffi.gemstone.GemPreferencesState

sealed interface PreferencesRowUIModel {
    data class Link(val model: ListItemModel, val action: PreferencesAction) : PreferencesRowUIModel
    data class Language(val model: ListItemModel) : PreferencesRowUIModel
    data class AppearancePicker(val model: ListItemModel, val current: Appearance) : PreferencesRowUIModel
    data class PerpetualsSwitch(val model: ListItemModel, val isEnabled: Boolean) : PreferencesRowUIModel
    data class Picker(val model: ListItemModel, val setting: PerpetualSetting, val current: Int, val options: List<PickerOption>) : PreferencesRowUIModel
}

enum class PerpetualSetting { Leverage, TakeProfit, StopLoss }

data class PickerOption(val value: Int, val label: String)

data class PerpetualOptions(
    val leverage: List<PickerOption>,
    val takeProfit: List<PickerOption>,
    val stopLoss: List<PickerOption>,
)

data class PerpetualValues(val isEnabled: Boolean, val leverage: Int, val takeProfit: Int, val stopLoss: Int)

internal fun GemPreferencesRow.uiModel(
    context: Context,
    state: GemPreferencesState,
    appearance: Appearance,
    perpetual: PerpetualValues,
    options: PerpetualOptions,
): PreferencesRowUIModel = when (this) {
    GemPreferencesRow.CURRENCY -> PreferencesRowUIModel.Link(listItem(context, subtitle = state.currency.text()), PreferencesAction.Currencies)
    GemPreferencesRow.LANGUAGE -> PreferencesRowUIModel.Language(listItem(context))
    GemPreferencesRow.APPEARANCE -> PreferencesRowUIModel.AppearancePicker(listItem(context, subtitle = context.getString(appearance.stringRes())), appearance)
    GemPreferencesRow.NETWORKS -> PreferencesRowUIModel.Link(listItem(context), PreferencesAction.Networks)
    GemPreferencesRow.CONTACTS -> PreferencesRowUIModel.Link(listItem(context), PreferencesAction.Contacts)
    GemPreferencesRow.PERPETUALS -> PreferencesRowUIModel.PerpetualsSwitch(listItem(context), perpetual.isEnabled)
    GemPreferencesRow.PERPETUAL_LEVERAGE -> PreferencesRowUIModel.Picker(listItem(context, subtitle = options.leverage.label(perpetual.leverage)), PerpetualSetting.Leverage, perpetual.leverage, options.leverage)
    GemPreferencesRow.PERPETUAL_TAKE_PROFIT -> PreferencesRowUIModel.Picker(listItem(context, subtitle = options.takeProfit.label(perpetual.takeProfit)), PerpetualSetting.TakeProfit, perpetual.takeProfit, options.takeProfit)
    GemPreferencesRow.PERPETUAL_STOP_LOSS -> PreferencesRowUIModel.Picker(listItem(context, subtitle = options.stopLoss.label(perpetual.stopLoss)), PerpetualSetting.StopLoss, perpetual.stopLoss, options.stopLoss)
}

private fun GemPreferencesRow.listItem(context: Context, subtitle: String? = null): ListItemModel = ListItemModel(
    title = context.getString(stringRes()),
    subtitle = subtitle,
    image = icon()?.let { ListItemImage.Drawable(it) },
)

private fun List<PickerOption>.label(value: Int): String? = firstOrNull { it.value == value }?.label
