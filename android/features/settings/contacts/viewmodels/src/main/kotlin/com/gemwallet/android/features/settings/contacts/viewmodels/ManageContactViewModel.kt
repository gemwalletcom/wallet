package com.gemwallet.android.features.settings.contacts.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.contacts.cases.AddContactAddress
import com.gemwallet.android.application.contacts.cases.GetContacts
import com.gemwallet.android.application.contacts.cases.SaveContact
import com.gemwallet.android.application.recipient.cases.GetNameRecord
import com.gemwallet.android.ext.decodePayment
import com.gemwallet.android.ext.isValidAddress
import com.gemwallet.android.ext.request
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactAddressForm
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactAddressInput
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactAvatarState
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ManageContactPage
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ManageContactState
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ManageContactUIState
import com.gemwallet.android.ui.components.image.EmojiAvatarRenderer
import com.gemwallet.android.ui.models.name.AddressInputModel
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactAddress
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemContactAvatar
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.GemNameServiceInterface
import java.util.UUID
import javax.inject.Inject

@HiltViewModel
class ManageContactViewModel @Inject constructor(
    private val getContacts: GetContacts,
    private val saveContactCase: SaveContact,
    private val addContactAddress: AddContactAddress,
    @param:ApplicationContext private val context: Context,
    private val nameService: GemNameServiceInterface,
    private val paymentService: GemPaymentService,
    getNameRecord: GetNameRecord,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private sealed interface Mode {
        data object Add : Mode
        data class Edit(val contactId: String) : Mode
    }

    private val mode: Mode = run {
        val editContactId = savedStateHandle.get<String>(RouteArgument.ContactId.key)
        if (editContactId != null) Mode.Edit(editContactId) else Mode.Add
    }
    private val contactId: String = (mode as? Mode.Edit)?.contactId ?: UUID.randomUUID().toString()
    private var contact: Contact? = null

    private val addressInput = AddressInputModel(
        getNameRecord = getNameRecord,
        nameService = nameService,
        scope = viewModelScope,
    )

    private val state = MutableStateFlow(ManageContactState(isEdit = mode is Mode.Edit))
    val uiState: StateFlow<ManageContactUIState> = combine(
        state,
        addressInput.text,
        addressInput.nameResolveState,
        addressInput.showError,
        addressInput.isValid,
    ) { current, address, resolve, showError, isValid ->
        ManageContactUIState(
            isEdit = current.isEdit,
            name = current.name,
            description = current.description,
            avatar = current.avatar,
            addresses = current.addresses,
            page = current.page,
            isSaving = current.isSaving,
            saved = current.saved,
            addressInput = current.form?.let { form ->
                ContactAddressInput(
                    editingId = form.editingId,
                    chain = form.chain,
                    memo = form.memo,
                    address = address,
                    nameResolveState = resolve,
                    isAddressValid = isValid,
                    showAddressError = showError,
                )
            },
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ManageContactUIState(isEdit = mode is Mode.Edit))

    init {
        when (val mode = mode) {
            is Mode.Edit -> viewModelScope.launch(Dispatchers.IO) {
                val data = getContacts.getContact(mode.contactId) ?: return@launch
                contact = data.contact
                state.update {
                    it.copy(
                        name = data.contact.name,
                        description = data.contact.description ?: "",
                        avatar = ContactAvatarState.from(data.contact.imageUrl),
                        addresses = data.addresses,
                    )
                }
            }
            Mode.Add -> Unit
        }
    }

    fun setName(value: String) = state.update { it.copy(name = value) }

    fun setDescription(value: String) = state.update { it.copy(description = value) }

    fun selectAvatar() = state.update { it.copy(page = ManageContactPage.Avatar) }

    fun cancelAvatar() = state.update { it.copy(page = ManageContactPage.Form) }

    fun setAvatar(emoji: String, backgroundColor: Int) = state.update {
        it.copy(avatar = ContactAvatarState.Emoji(emoji, backgroundColor), page = ManageContactPage.Form)
    }

    fun removeAvatar() = state.update { it.copy(avatar = ContactAvatarState.Empty) }

    fun deleteAddress(address: ContactAddress) = state.update {
        it.copy(addresses = it.addresses.filterNot { item -> item.id == address.id })
    }

    fun addAddress() {
        val form = ContactAddressForm(chain = addContactAddress.defaultChain())
        addressInput.reset()
        addressInput.setChain(form.chain)
        state.update { it.copy(page = ManageContactPage.Address, form = form) }
    }

    fun editAddress(address: ContactAddress) {
        addressInput.reset()
        addressInput.setChain(address.chain)
        addressInput.onTextChange(address.address)
        state.update {
            it.copy(
                page = ManageContactPage.Address,
                form = ContactAddressForm(
                    editingId = address.id,
                    chain = address.chain,
                    memo = address.memo ?: "",
                ),
            )
        }
    }

    fun cancelAddress() {
        addressInput.reset()
        state.update { it.copy(page = ManageContactPage.Form) }
    }

    fun setAddress(value: String) = addressInput.onTextChange(value)

    fun setMemo(value: String) = updateInput { it.copy(memo = value) }

    fun scanAddress(data: String) = applyExternalAddress(data)

    fun pasteAddress(data: String) = applyExternalAddress(data)

    private fun applyExternalAddress(data: String) {
        val decoded = paymentService.decodePayment(data)?.request
        val address = (decoded?.address?.ifBlank { null } ?: data).trim()
        val memo = decoded?.memo
        addressInput.applyExternalAddress(address)
        updateInput { it.copy(memo = memo ?: it.memo) }
    }

    fun selectChain() = state.update { it.copy(page = ManageContactPage.SelectChain) }

    fun cancelSelectChain() = state.update { it.copy(page = ManageContactPage.Address) }

    fun setChain(chain: Chain) {
        addressInput.setChain(chain)
        state.update {
            it.copy(page = ManageContactPage.Address, form = it.form?.copy(chain = chain, memo = ""))
        }
    }

    private fun updateInput(transform: (ContactAddressForm) -> ContactAddressForm) = state.update { current ->
        val form = current.form ?: return@update current
        current.copy(form = transform(form))
    }

    fun confirmAddress() {
        val input = uiState.value.addressInput ?: return
        if (!input.isConfirmEnabled) return

        val address = addressInput.resolvedAddress
        addressInput.reset()
        state.update { current ->
            current.copy(
                addresses = addContactAddress.addAddress(
                    addresses = current.addresses,
                    contactId = contactId,
                    chain = input.chain,
                    address = address,
                    memo = input.memo,
                    replacingId = input.editingId,
                ),
                page = ManageContactPage.Form,
            )
        }
    }

    fun save() {
        val current = uiState.value
        if (!current.isSaveEnabled) return
        state.update { it.copy(isSaving = true) }

        viewModelScope.launch(Dispatchers.IO) {
            saveContactCase.saveContact(
                id = contactId,
                existing = contact,
                name = current.name,
                description = current.description,
                avatar = when (val avatar = current.avatar) {
                    ContactAvatarState.Empty -> GemContactAvatar.Empty
                    is ContactAvatarState.Image -> GemContactAvatar.Image(avatar.imageUrl)
                    is ContactAvatarState.Emoji -> GemContactAvatar.Rendered(
                        EmojiAvatarRenderer.render(context, avatar.emoji, avatar.backgroundColor)
                    )
                },
                addresses = current.addresses,
            )
            state.update { it.copy(saved = true) }
        }
    }
}
