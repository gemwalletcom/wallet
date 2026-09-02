package com.gemwallet.android.features.settings.contacts.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import android.util.Log
import com.gemwallet.android.application.contacts.cases.GetContacts
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.isValidAddress
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
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemContactAddressInput
import uniffi.gemstone.GemContactAvatar
import uniffi.gemstone.GemContactInput
import uniffi.gemstone.GemManageContactServiceInterface
import uniffi.gemstone.GemNameServiceInterface
import java.util.UUID
import javax.inject.Inject

@HiltViewModel
class ManageContactViewModel @Inject constructor(
    private val getContacts: GetContacts,
    @param:ApplicationContext private val context: Context,
    private val service: GemManageContactServiceInterface,
    nameService: GemNameServiceInterface,
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

    private val addressInput = AddressInputModel(nameService, viewModelScope)

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
        val form = ContactAddressForm(chain = service.defaultChain().toChain() ?: Chain.Bitcoin)
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
        val scan = service.scannedAddress(data)
        addressInput.applyExternalAddress(scan.address)
        updateInput { it.copy(memo = scan.memo ?: it.memo) }
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
                addresses = GemContactAddressInput(
                    contactId = contactId,
                    chain = input.chain.string,
                    address = address,
                    memo = input.memo,
                    replacingId = input.editingId,
                ).addAddress(current.addresses.map { it.toJson() }).map { it.decodeJson<ContactAddress>() },
                page = ManageContactPage.Form,
            )
        }
    }

    fun save() {
        val current = uiState.value
        if (!current.isSaveEnabled) return
        state.update { it.copy(isSaving = true) }

        viewModelScope.launch(Dispatchers.IO) {
            val input = GemContactInput(
                id = contactId,
                existing = contact?.toJson(),
                name = current.name,
                description = current.description,
                avatar = when (val avatar = current.avatar) {
                    ContactAvatarState.Empty -> GemContactAvatar.Empty
                    is ContactAvatarState.Image -> GemContactAvatar.Image(avatar.imageUrl)
                    is ContactAvatarState.Emoji -> GemContactAvatar.Rendered(
                        EmojiAvatarRenderer.render(context, avatar.emoji, avatar.backgroundColor)
                    )
                },
                addresses = current.addresses.map { it.toJson() },
            )
            runCatchingCancellable { service.saveContact(input) }
                .onSuccess { state.update { it.copy(saved = true) } }
                .onFailure { error ->
                    Log.e(TAG, "saving contact $contactId failed", error)
                    state.update { it.copy(isSaving = false) }
                }
        }
    }

    private companion object {
        const val TAG = "ManageContact"
    }
}
