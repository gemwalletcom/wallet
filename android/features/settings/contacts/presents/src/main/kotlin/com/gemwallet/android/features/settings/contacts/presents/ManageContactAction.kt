package com.gemwallet.android.features.settings.contacts.presents

import com.wallet.core.primitives.ContactAddress

sealed interface ManageContactAction {
    data object AddAddress : ManageContactAction
    data class EditAddress(val address: ContactAddress) : ManageContactAction
    data class DeleteAddress(val address: ContactAddress) : ManageContactAction
    data object Save : ManageContactAction
    data object Cancel : ManageContactAction
}
