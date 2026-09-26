package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.contacts.presents.ContactEditorScreen
import com.gemwallet.android.features.contacts.presents.ContactsAction
import com.gemwallet.android.features.contacts.presents.ContactsScreen
import com.gemwallet.android.ui.navigation.contactIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import kotlinx.serialization.Serializable

@Serializable
data object ContactsRoute : NavKey

@Serializable
data object AddContactRoute : NavKey

@Serializable
data class EditContactRoute(val contactId: String) : NavKey

fun EntryProviderScope<NavKey>.contactsScreen(onAction: (ContactsAction) -> Unit) {
    val onCancel = { onAction(ContactsAction.Cancel) }

    entry<ContactsRoute> {
        ContactsScreen(onAction = onAction)
    }

    entry<AddContactRoute> {
        ContactEditorScreen(
            onSaved = onCancel,
            onCancel = onCancel,
        )
    }

    entry<EditContactRoute>(
        metadata = { key -> routeArguments(contactIdArgument(key.contactId)) },
    ) {
        ContactEditorScreen(
            onSaved = onCancel,
            onCancel = onCancel,
        )
    }
}
