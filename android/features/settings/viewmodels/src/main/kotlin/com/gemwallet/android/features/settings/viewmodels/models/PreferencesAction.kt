package com.gemwallet.android.features.settings.viewmodels.models

import com.wallet.core.primitives.Appearance

sealed interface PreferencesAction {
    sealed interface Navigation : PreferencesAction

    data object Currencies : Navigation
    data object Networks : Navigation
    data object Contacts : Navigation
    data object Cancel : Navigation

    data class SetAppearance(val appearance: Appearance) : PreferencesAction
    data class SetPerpetualEnabled(val enabled: Boolean) : PreferencesAction
    data class SetPerpetualOption(val setting: PerpetualSetting, val value: Int) : PreferencesAction
}
