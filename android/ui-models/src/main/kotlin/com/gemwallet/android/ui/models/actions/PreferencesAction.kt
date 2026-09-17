package com.gemwallet.android.ui.models.actions

sealed interface PreferencesAction {
    data object Currencies : PreferencesAction
    data object Networks : PreferencesAction
    data object Contacts : PreferencesAction
    data object Cancel : PreferencesAction
}
