package com.gemwallet.android.features.settings.contacts.presents

sealed interface ContactAddressEditorAction {
    data object SelectChain : ContactAddressEditorAction
    data object Confirm : ContactAddressEditorAction
    data object Cancel : ContactAddressEditorAction
}
