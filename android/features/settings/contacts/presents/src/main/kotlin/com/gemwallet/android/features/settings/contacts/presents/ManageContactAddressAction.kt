package com.gemwallet.android.features.settings.contacts.presents

sealed interface ManageContactAddressAction {
    data object SelectChain : ManageContactAddressAction
    data object Confirm : ManageContactAddressAction
    data object Cancel : ManageContactAddressAction
}
