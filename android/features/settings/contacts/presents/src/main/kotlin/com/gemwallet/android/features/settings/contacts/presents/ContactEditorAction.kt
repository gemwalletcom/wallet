package com.gemwallet.android.features.settings.contacts.presents

import com.wallet.core.primitives.ContactAddress

sealed interface ContactEditorAction {
    data object SelectAvatar : ContactEditorAction
    data object RemoveAvatar : ContactEditorAction
    data object AddAddress : ContactEditorAction
    data class EditAddress(val address: ContactAddress) : ContactEditorAction
    data class DeleteAddress(val address: ContactAddress) : ContactEditorAction
    data object Save : ContactEditorAction
    data object Cancel : ContactEditorAction
}
