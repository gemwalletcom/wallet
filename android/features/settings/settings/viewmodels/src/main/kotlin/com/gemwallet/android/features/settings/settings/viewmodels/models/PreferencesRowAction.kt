package com.gemwallet.android.features.settings.settings.viewmodels.models

import com.gemwallet.android.ui.models.actions.PreferencesAction
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemPerpetualDefaults
import uniffi.gemstone.GemPerpetualPickers
import uniffi.gemstone.GemPickerOption

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

fun GemListRow.preferencesAction(): PreferencesRowAction? = when (this) {
    is GemListRow.Link -> when (title) {
        GemListRowTitle.CURRENCY -> PreferencesRowAction.Open(PreferencesAction.Currencies)
        GemListRowTitle.NETWORKS -> PreferencesRowAction.Open(PreferencesAction.Networks)
        GemListRowTitle.CONTACTS -> PreferencesRowAction.Open(PreferencesAction.Contacts)
        GemListRowTitle.LANGUAGE -> PreferencesRowAction.Language
        GemListRowTitle.APPEARANCE -> PreferencesRowAction.Appearance
        else -> null
    }

    is GemListRow.Toggle -> when (title) {
        GemListRowTitle.PERPETUALS -> PreferencesRowAction.Perpetuals(isOn)
        else -> null
    }

    is GemListRow.Picker -> when (title) {
        GemListRowTitle.PERPETUAL_LEVERAGE -> PreferencesRowAction.Option(PerpetualSetting.Leverage)
        GemListRowTitle.PERPETUAL_TAKE_PROFIT -> PreferencesRowAction.Option(PerpetualSetting.TakeProfit)
        GemListRowTitle.PERPETUAL_STOP_LOSS -> PreferencesRowAction.Option(PerpetualSetting.StopLoss)
        else -> null
    }

    else -> null
}
