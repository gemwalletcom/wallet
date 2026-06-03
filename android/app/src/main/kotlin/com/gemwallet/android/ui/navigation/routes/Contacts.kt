package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.settings.contacts.presents.ContactsAction
import com.gemwallet.android.features.settings.contacts.presents.ContactsNavScreen
import com.gemwallet.android.features.settings.contacts.presents.ManageContactNavScreen
import com.gemwallet.android.model.ChainRecipient
import com.gemwallet.android.ui.navigation.addressArgument
import com.gemwallet.android.ui.navigation.chainArgument
import com.gemwallet.android.ui.navigation.contactIdArgument
import com.gemwallet.android.ui.navigation.memoArgument
import com.gemwallet.android.ui.navigation.routeArguments
import kotlinx.serialization.Serializable

@Serializable
data object ContactsRoute : NavKey

@Serializable
data object AddContactRoute : NavKey

@Serializable
data class AddContactWithAddressRoute(val recipient: ChainRecipient) : NavKey

@Serializable
data class EditContactRoute(val contactId: String) : NavKey

fun EntryProviderScope<NavKey>.contactsScreen(
    onAction: (ContactsAction) -> Unit,
) {
    val onCancel = { onAction(ContactsAction.Cancel) }

    entry<ContactsRoute> {
        ContactsNavScreen(onAction = onAction)
    }

    entry<AddContactRoute> {
        ManageContactNavScreen(
            onSaved = onCancel,
            onCancel = onCancel,
        )
    }

    entry<AddContactWithAddressRoute>(
        metadata = { key ->
            routeArguments(
                chainArgument(key.recipient.chain.string),
                addressArgument(key.recipient.address),
                memoArgument(key.recipient.memo),
            )
        },
    ) {
        ManageContactNavScreen(
            onSaved = onCancel,
            onCancel = onCancel,
        )
    }

    entry<EditContactRoute>(
        metadata = { key -> routeArguments(contactIdArgument(key.contactId)) },
    ) {
        ManageContactNavScreen(
            onSaved = onCancel,
            onCancel = onCancel,
        )
    }
}
