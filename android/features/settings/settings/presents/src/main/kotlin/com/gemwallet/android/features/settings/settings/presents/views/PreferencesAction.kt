package com.gemwallet.android.features.settings.settings.presents.views

sealed interface PreferencesAction {
    data object Currencies : PreferencesAction
    data object Networks : PreferencesAction
    data object Contacts : PreferencesAction
    data object Cancel : PreferencesAction
}
