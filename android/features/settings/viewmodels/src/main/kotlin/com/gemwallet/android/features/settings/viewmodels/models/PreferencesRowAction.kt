package com.gemwallet.android.features.settings.viewmodels.models

import com.gemwallet.android.ui.models.actions.PreferencesAction
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemPerpetualDefaults
import uniffi.gemstone.GemPerpetualPickers
import uniffi.gemstone.GemPickerOption
import uniffi.gemstone.GemRowAction

sealed interface PreferencesRowAction {
    data class Open(val action: PreferencesAction) : PreferencesRowAction
    data object Language : PreferencesRowAction
    data object Appearance : PreferencesRowAction
    data class Perpetuals(val isOn: Boolean) : PreferencesRowAction
    data class Option(val setting: PerpetualSetting) : PreferencesRowAction
}

fun GemListRow.opensPicker(): Boolean = this is GemListRow.Link

enum class PerpetualSetting { Leverage, TakeProfit, StopLoss }

fun GemPerpetualPickers.of(setting: PerpetualSetting): List<GemPickerOption> = when (setting) {
    PerpetualSetting.Leverage -> leverage
    PerpetualSetting.TakeProfit -> takeProfit
    PerpetualSetting.StopLoss -> stopLoss
}

fun GemPerpetualDefaults.value(setting: PerpetualSetting): Int = when (setting) {
    PerpetualSetting.Leverage -> leverage.toInt()
    PerpetualSetting.TakeProfit -> takeProfitPercent.toInt()
    PerpetualSetting.StopLoss -> stopLossPercent.toInt()
}

fun GemListRow.preferencesAction(): PreferencesRowAction? = when (action()) {
    GemRowAction.Currency -> PreferencesRowAction.Open(PreferencesAction.Currencies)
    GemRowAction.Networks -> PreferencesRowAction.Open(PreferencesAction.Networks)
    GemRowAction.Contacts -> PreferencesRowAction.Open(PreferencesAction.Contacts)
    GemRowAction.Language -> PreferencesRowAction.Language
    GemRowAction.Appearance -> PreferencesRowAction.Appearance
    GemRowAction.Perpetuals -> (this as? GemListRow.Toggle)?.let { PreferencesRowAction.Perpetuals(it.isOn) }
    GemRowAction.PerpetualLeverage -> PreferencesRowAction.Option(PerpetualSetting.Leverage)
    GemRowAction.PerpetualTakeProfit -> PreferencesRowAction.Option(PerpetualSetting.TakeProfit)
    GemRowAction.PerpetualStopLoss -> PreferencesRowAction.Option(PerpetualSetting.StopLoss)
    else -> null
}
