package com.gemwallet.android.features.settings.contacts.viewmodels.models

import uniffi.gemstone.GemContactAddressField
import uniffi.gemstone.contactAddressFields
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemNameRecordState
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ContactAddress

enum class ManageContactPage {
    Form,
    Address,
    SelectChain,
    Avatar,
}

data class ManageContactUIState(
    val isEdit: Boolean = false,
    val name: String = "",
    val description: String = "",
    val avatar: ContactAvatarState = ContactAvatarState.Empty,
    val addresses: List<ContactAddress> = emptyList(),
    val page: ManageContactPage = ManageContactPage.Form,
    val addressInput: ContactAddressInput? = null,
    val isSaving: Boolean = false,
    val saved: Boolean = false,
    val error: GemErrorText? = null,
    val isSaveEnabled: Boolean = false,
)

data class ManageContactState(
    val isEdit: Boolean = false,
    val name: String = "",
    val description: String = "",
    val avatar: ContactAvatarState = ContactAvatarState.Empty,
    val addresses: List<ContactAddress> = emptyList(),
    val page: ManageContactPage = ManageContactPage.Form,
    val form: ContactAddressForm? = null,
    val isSaving: Boolean = false,
    val saved: Boolean = false,
    val error: GemErrorText? = null,
)

data class ContactAddressForm(
    val editingId: String? = null,
    val chain: Chain,
    val memo: String = "",
)

data class ContactAddressInput(
    val editingId: String? = null,
    val chain: Chain,
    val address: String = "",
    val memo: String = "",
    val nameResolveState: GemNameRecordState = GemNameRecordState.None,
    val isAddressValid: Boolean = false,
    val showAddressError: Boolean = false,
) {
    val fields: List<GemContactAddressField>
        get() = contactAddressFields(chain.string)

    val isConfirmEnabled: Boolean
        get() = isAddressValid
}
