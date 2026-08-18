package com.gemwallet.android.features.settings.contacts.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.cases.contacts.AddContact
import com.gemwallet.android.cases.contacts.GetContacts
import com.gemwallet.android.cases.contacts.UpdateContact
import com.gemwallet.android.cases.name.ResolveName
import com.gemwallet.android.ext.isValidAddress
import com.gemwallet.android.ui.models.name.AddressInputModel
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactAddressForm
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactAddressInput
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ManageContactPage
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ManageContactState
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ManageContactUIState
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactAddress
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.util.UUID
import javax.inject.Inject

@HiltViewModel
class ManageContactViewModel @Inject constructor(
    private val getContacts: GetContacts,
    private val addContactCase: AddContact,
    private val updateContactCase: UpdateContact,
    resolveName: ResolveName,
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
    private var createdAt: Long? = null

    private val addressInput = AddressInputModel(
        resolveName = resolveName,
        scope = viewModelScope,
        validateAddress = { address, chain -> chain.isValidAddress(address) },
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
            addresses = current.addresses,
            page = current.page,
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
                createdAt = data.contact.createdAt
                state.update {
                    it.copy(
                        isEdit = true,
                        name = data.contact.name,
                        description = data.contact.description ?: "",
                        addresses = data.addresses,
                    )
                }
            }
            Mode.Add -> Unit
        }
    }

    fun setName(value: String) = state.update { it.copy(name = value) }

    fun setDescription(value: String) = state.update { it.copy(description = value) }

    fun deleteAddress(address: ContactAddress) = state.update {
        it.copy(addresses = it.addresses.filterNot { item -> item.id == address.id })
    }

    fun addAddress() {
        val form = ContactAddressForm()
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
        val decoded = runCatching { uniffi.gemstone.paymentDecodeUrl(data) }.getOrNull()
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

        val address = contactAddress(
            chain = input.chain,
            address = addressInput.resolvedAddress,
            memo = input.memo.ifBlank { null },
        )

        addressInput.reset()
        state.update { current ->
            current.copy(
                addresses = current.addresses.upsert(address, setOfNotNull(input.editingId, address.id)),
                page = ManageContactPage.Form,
            )
        }
    }

    private fun List<ContactAddress>.upsert(address: ContactAddress, replacing: Set<String>): List<ContactAddress> {
        val index = indexOfFirst { it.id in replacing }
        val without = filterNot { it.id in replacing }
        return if (index < 0) without + address else without.take(index) + address + without.drop(index)
    }

    private fun contactAddress(chain: Chain, address: String, memo: String?): ContactAddress = ContactAddress(
        id = "${contactId}_${chain.string}_${address}",
        contactId = contactId,
        address = address,
        chain = chain,
        memo = memo,
    )

    fun save() {
        val current = uiState.value
        if (!current.isSaveEnabled) return

        val now = System.currentTimeMillis()
        val contact = Contact(
            id = contactId,
            name = current.name.trim(),
            description = current.description.ifBlank { null },
            createdAt = createdAt ?: now,
            updatedAt = now,
        )

        viewModelScope.launch(Dispatchers.IO) {
            when (mode) {
                is Mode.Add -> addContactCase.addContact(contact, current.addresses)
                is Mode.Edit -> updateContactCase.updateContact(contact, current.addresses)
            }
            state.update { it.copy(saved = true) }
        }
    }
}
