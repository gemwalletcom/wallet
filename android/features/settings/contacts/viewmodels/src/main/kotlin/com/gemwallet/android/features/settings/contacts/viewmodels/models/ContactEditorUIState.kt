package com.gemwallet.android.features.settings.contacts.viewmodels.models

import com.gemwallet.android.ui.components.fields.NameResolveIndicatorUIModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ContactAddress
import uniffi.gemstone.GemContactAddressField
import uniffi.gemstone.GemContactSession
import uniffi.gemstone.contactAddressFields

enum class ContactEditorPage {
    Form,
    Address,
    SelectChain,
    Avatar,
}

data class ContactEditorUIState(
    val isEdit: Boolean = false,
    val name: String = "",
    val initials: String = "",
    val description: String = "",
    val avatar: ContactAvatarState = ContactAvatarState.Empty,
    val addresses: List<ContactAddress> = emptyList(),
    val addressRows: List<ContactAddressRowUIModel> = emptyList(),
    val addAddressListItem: ListItemModel? = null,
    val page: ContactEditorPage = ContactEditorPage.Form,
    val addressInput: ContactAddressInput? = null,
    val isSaving: Boolean = false,
    val saved: Boolean = false,
    val errorText: String? = null,
    val isSaveEnabled: Boolean = false,
)

data class ContactEditorState(
    val session: GemContactSession,
    val isEdit: Boolean,
    val emojiBackground: Int = 0,
    val page: ContactEditorPage = ContactEditorPage.Form,
    val form: ContactAddressForm? = null,
    val saved: Boolean = false,
    val errorText: String? = null,
)

data class ContactAddressForm(val editingId: String? = null, val chain: Chain, val memo: String = "")

data class ContactAddressInput(
    val editingId: String? = null,
    val chain: Chain,
    val address: String = "",
    val memo: String = "",
    val nameResolveIndicator: NameResolveIndicatorUIModel? = null,
    val isAddressValid: Boolean = false,
    val addressError: String = "",
) {
    val showsMemo: Boolean
        get() = GemContactAddressField.MEMO in contactAddressFields(chain.string)

    val isConfirmEnabled: Boolean
        get() = isAddressValid
}
