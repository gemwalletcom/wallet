package com.gemwallet.android.features.contacts.presents

import com.gemwallet.android.ui.models.navigation.ContactAddressDraft

sealed interface ContactsAction {
    data class OpenContact(val contactId: String) : ContactsAction
    data class AddContact(val draft: ContactAddressDraft?) : ContactsAction
    data object Cancel : ContactsAction
}
