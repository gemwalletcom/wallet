package com.gemwallet.android.features.settings.contacts.presents

sealed interface ContactsAction {
    data class OpenContact(val contactId: String) : ContactsAction
    data object AddContact : ContactsAction
    data object Cancel : ContactsAction
}
