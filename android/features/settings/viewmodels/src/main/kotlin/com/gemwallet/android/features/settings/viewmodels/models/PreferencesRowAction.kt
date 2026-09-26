package com.gemwallet.android.features.settings.viewmodels.models

import com.gemwallet.android.ui.models.actions.PreferencesAction
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemPerpetualDefaults
import uniffi.gemstone.GemPerpetualPickers
import uniffi.gemstone.GemPickerOption
import uniffi.gemstone.GemRowTap

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

fun GemListRow.preferencesAction(): PreferencesRowAction? = when (tap()) {
    GemRowTap.Currency -> PreferencesRowAction.Open(PreferencesAction.Currencies)
    GemRowTap.Networks -> PreferencesRowAction.Open(PreferencesAction.Networks)
    GemRowTap.Contacts -> PreferencesRowAction.Open(PreferencesAction.Contacts)
    GemRowTap.Language -> PreferencesRowAction.Language
    GemRowTap.Appearance -> PreferencesRowAction.Appearance
    GemRowTap.Perpetuals -> (this as? GemListRow.Toggle)?.let { PreferencesRowAction.Perpetuals(it.isOn) }
    GemRowTap.PerpetualLeverage -> PreferencesRowAction.Option(PerpetualSetting.Leverage)
    GemRowTap.PerpetualTakeProfit -> PreferencesRowAction.Option(PerpetualSetting.TakeProfit)
    GemRowTap.PerpetualStopLoss -> PreferencesRowAction.Option(PerpetualSetting.StopLoss)
    else -> null
}
