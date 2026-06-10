package com.gemwallet.android.features.settings.contacts.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.blockchain.operators.ValidateAddressOperator
import com.gemwallet.android.blockchain.operators.gemstone.checksumAddress
import com.gemwallet.android.cases.contacts.AddContact
import com.gemwallet.android.cases.contacts.GetContacts
import com.gemwallet.android.cases.contacts.UpdateContact
import com.gemwallet.android.cases.name.ResolveName
import com.gemwallet.android.features.recipient.viewmodel.NameRecordState
import com.gemwallet.android.features.recipient.viewmodel.NameResolveController
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactAddressInput
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ManageContactPage
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ManageContactUIState
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactAddress
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.util.UUID
import javax.inject.Inject

@HiltViewModel
class ManageContactViewModel @Inject constructor(
    private val getContacts: GetContacts,
    private val addContactCase: AddContact,
    private val updateContactCase: UpdateContact,
    private val validateAddress: ValidateAddressOperator,
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

    private val resolver = NameResolveController(resolveName, viewModelScope)

    private val state = MutableStateFlow(ManageContactUIState(isEdit = mode is Mode.Edit))
    val uiState: StateFlow<ManageContactUIState> = state.asStateFlow()

    init {
        viewModelScope.launch {
            resolver.state.collect { resolve ->
                updateInput { it.copy(nameResolveState = resolve) }
            }
        }
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
        resolver.reset()
        state.update { it.copy(page = ManageContactPage.Address, addressInput = ContactAddressInput()) }
    }

    fun editAddress(address: ContactAddress) {
        resolver.reset()
        state.update {
            it.copy(
                page = ManageContactPage.Address,
                addressInput = ContactAddressInput(
                    editingId = address.id,
                    chain = address.chain,
                    address = address.address,
                    memo = address.memo ?: "",
                ).validated(),
            )
        }
    }

    fun cancelAddress() {
        resolver.reset()
        state.update { it.copy(page = ManageContactPage.Form) }
    }

    fun setAddress(value: String) {
        updateInput { it.copy(address = value, showAddressError = false) }
        resolver.onInput(value, state.value.addressInput?.chain)
    }

    fun setMemo(value: String) = updateInput { it.copy(memo = value) }

    fun scanAddress(data: String) = applyExternalAddress(data)

    fun pasteAddress(data: String) = applyExternalAddress(data)

    private fun applyExternalAddress(data: String) {
        resolver.reset()
        val decoded = runCatching { uniffi.gemstone.paymentDecodeUrl(data) }.getOrNull()
        val address = (decoded?.address?.ifBlank { null } ?: data).trim()
        val memo = decoded?.memo
        state.update {
            val input = it.addressInput ?: return@update it
            it.copy(
                addressInput = input
                    .copy(address = address, nameResolveState = NameRecordState.None, memo = memo ?: input.memo)
                    .validated()
                    .withAddressError(),
            )
        }
    }

    fun selectChain() = state.update { it.copy(page = ManageContactPage.SelectChain) }

    fun cancelSelectChain() = state.update { it.copy(page = ManageContactPage.Address) }

    fun setChain(chain: Chain) {
        resolver.reset()
        state.update {
            it.copy(
                page = ManageContactPage.Address,
                addressInput = it.addressInput
                    ?.copy(chain = chain, memo = "", nameResolveState = NameRecordState.None)
                    ?.validated()
                    ?.withAddressError(),
            )
        }
    }

    private fun updateInput(transform: (ContactAddressInput) -> ContactAddressInput) = state.update { current ->
        val input = current.addressInput ?: return@update current
        current.copy(addressInput = transform(input).validated())
    }

    private fun ContactAddressInput.validated(): ContactAddressInput = copy(
        isAddressValid = resolvedAddress.isNotBlank() && validateAddress(resolvedAddress, chain).getOrNull() == true,
    )

    private fun ContactAddressInput.withAddressError(): ContactAddressInput = copy(
        showAddressError = resolvedAddress.isNotBlank() && !isAddressValid,
    )

    fun confirmAddress() {
        val input = state.value.addressInput ?: return
        if (!input.isConfirmEnabled) return

        val address = contactAddress(
            chain = input.chain,
            address = input.chain.checksumAddress(input.resolvedAddress),
            memo = input.memo.ifBlank { null },
        )

        resolver.reset()
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
        val current = state.value
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
