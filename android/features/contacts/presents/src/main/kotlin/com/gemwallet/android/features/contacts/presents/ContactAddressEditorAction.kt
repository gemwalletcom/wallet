package com.gemwallet.android.features.contacts.presents

sealed interface ContactAddressEditorAction {
    data class SetAddress(val address: String) : ContactAddressEditorAction
    data class SetMemo(val memo: String) : ContactAddressEditorAction
    data class Scan(val address: String) : ContactAddressEditorAction
    data class Paste(val address: String) : ContactAddressEditorAction
    data object SelectChain : ContactAddressEditorAction
    data object ValidateAddress : ContactAddressEditorAction
    data object Confirm : ContactAddressEditorAction
    data object Cancel : ContactAddressEditorAction
}
