package com.gemwallet.android.features.settings.contacts.viewmodels.models

import com.gemwallet.android.ext.checksumAddress
import com.gemwallet.android.ext.isMemoSupport
import com.gemwallet.android.features.recipient.viewmodel.NameRecordState
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ContactAddress

enum class ManageContactPage {
    Form,
    Address,
    SelectChain,
}

data class ManageContactUIState(
    val isEdit: Boolean = false,
    val name: String = "",
    val description: String = "",
    val addresses: List<ContactAddress> = emptyList(),
    val page: ManageContactPage = ManageContactPage.Form,
    val addressInput: ContactAddressInput? = null,
    val saved: Boolean = false,
) {
    val isSaveEnabled: Boolean
        get() = name.trim().isNotEmpty()
}

data class ContactAddressInput(
    val editingId: String? = null,
    val chain: Chain = Chain.Bitcoin,
    val address: String = "",
    val nameResolveState: NameRecordState = NameRecordState.None,
    val isAddressValid: Boolean = false,
    val showAddressError: Boolean = false,
    val memo: String = "",
) {
    val resolvedAddress: String
        get() = chain.checksumAddress(nameResolveState.nameRecord?.address?.takeIf { it.isNotEmpty() } ?: address)

    val showMemo: Boolean
        get() = chain.isMemoSupport()

    val isConfirmEnabled: Boolean
        get() = nameResolveState !is NameRecordState.Loading && isAddressValid
}
