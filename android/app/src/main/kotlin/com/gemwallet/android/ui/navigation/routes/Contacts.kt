package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.contacts.presents.ContactEditorScreen
import com.gemwallet.android.features.contacts.presents.ContactsAction
import com.gemwallet.android.features.contacts.presents.ContactsScreen
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.contactIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.Chain
import kotlinx.serialization.Serializable

@Serializable
data object ContactsRoute : NavKey

@Serializable
data class AddContactRoute(val chain: Chain? = null, val address: String? = null, val memo: String? = null) : NavKey

@Serializable
data class AddToContactRoute(val chain: Chain, val address: String, val memo: String? = null) : NavKey

@Serializable
data class EditContactRoute(val contactId: String) : NavKey

fun EntryProviderScope<NavKey>.contactsScreen(onAction: (ContactsAction) -> Unit) {
    val onCancel = { onAction(ContactsAction.Cancel) }

    entry<ContactsRoute> {
        ContactsScreen(onAction = onAction)
    }

    entry<AddToContactRoute>(
        metadata = { key -> routeArguments(RouteArgument.Chain to key.chain.string, RouteArgument.Address to key.address, RouteArgument.Memo to key.memo) },
    ) {
        ContactsScreen(onAction = onAction)
    }

    entry<AddContactRoute>(
        metadata = { key -> routeArguments(RouteArgument.Chain to key.chain?.string, RouteArgument.Address to key.address, RouteArgument.Memo to key.memo) },
    ) {
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
