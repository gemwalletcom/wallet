package com.gemwallet.android.features.contacts.viewmodels.models

import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemContactAddressRow
import uniffi.gemstone.GemNameIndicator

enum class ContactEditorPage {
    Form,
    Address,
    SelectChain,
    Avatar,
}

data class ContactEditorUIState(
    val isEdit: Boolean = false,
    val name: String = "",
    val description: String = "",
    val avatar: ListItemImage = ListItemImage.Initials(""),
    val hasAvatar: Boolean = false,
    val addressRows: List<GemContactAddressRow> = emptyList(),
    val page: ContactEditorPage = ContactEditorPage.Form,
    val addressInput: ContactAddressInput? = null,
    val isSaving: Boolean = false,
    val saved: Boolean = false,
    val errorText: String? = null,
    val isSaveEnabled: Boolean = false,
)

data class ContactAddressInput(
    val chain: Chain,
    val address: String = "",
    val memo: String = "",
    val nameResolveIndicator: GemNameIndicator? = null,
    val isAddressValid: Boolean = false,
    val addressError: String = "",
    val showsMemo: Boolean = false,
)
