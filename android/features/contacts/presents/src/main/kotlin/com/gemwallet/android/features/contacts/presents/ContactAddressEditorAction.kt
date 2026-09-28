package com.gemwallet.android.features.contacts.presents

sealed interface ContactAddressEditorAction {
    data object SelectChain : ContactAddressEditorAction
    data object Confirm : ContactAddressEditorAction
    data object Cancel : ContactAddressEditorAction
}
