package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.contacts.presents.ContactEditorNavScreen
import com.gemwallet.android.features.contacts.presents.ContactsAction
import com.gemwallet.android.features.contacts.presents.ContactsNavScreen
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
        ContactsNavScreen(onAction = onAction)
    }

    entry<AddContactRoute> {
        ContactEditorNavScreen(
            onSaved = onCancel,
            onCancel = onCancel,
        )
    }

    entry<EditContactRoute>(
        metadata = { key -> routeArguments(contactIdArgument(key.contactId)) },
    ) {
        ContactEditorNavScreen(
            onSaved = onCancel,
            onCancel = onCancel,
        )
    }
}
