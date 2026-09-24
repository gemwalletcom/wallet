package com.gemwallet.android.features.settings.contacts.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.contacts.cases.GetContacts
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactAddressForm
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactAddressInput
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactEditorPage
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactEditorState
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactEditorUIState
import com.gemwallet.android.features.settings.contacts.viewmodels.models.addAddressListItem
import com.gemwallet.android.features.settings.contacts.viewmodels.models.listItemImage
import com.gemwallet.android.features.settings.contacts.viewmodels.models.rows
import com.gemwallet.android.ui.components.image.EmojiAvatarRenderer
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.name.AddressInputModel
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.style.indicator
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ContactAddress
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemContactAddressField
import uniffi.gemstone.GemContactAddressInput
import uniffi.gemstone.GemContactAvatar
import uniffi.gemstone.GemContactAvatarChoice
import uniffi.gemstone.GemContactEditorServiceInterface
import uniffi.gemstone.GemContactSession
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.contactAddressFields
import javax.inject.Inject

@HiltViewModel
class ContactEditorViewModel @Inject constructor(
    private val getContacts: GetContacts,
    @param:ApplicationContext private val context: Context,
    private val service: GemContactEditorServiceInterface,
    nameService: GemNameServiceInterface,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    private sealed interface Mode {
        data object Add : Mode
        data class Edit(val contactId: String) : Mode
    }

    private val mode: Mode = run {
        val editContactId = savedStateHandle.get<String>(RouteArgument.ContactId.key)
        if (editContactId != null) Mode.Edit(editContactId) else Mode.Add
    }
    private val addressInput = AddressInputModel(nameService, viewModelScope)

    private val state = MutableStateFlow(
        ContactEditorState(
            session = service.newSession(null, emptyList()).let { session ->
                (mode as? Mode.Edit)?.let { session.copy(id = it.contactId) } ?: session
            },
            isEdit = mode is Mode.Edit,
        ),
    )
    val uiState: StateFlow<ContactEditorUIState> = combine(
        state,
        addressInput.text,
        addressInput.nameResolveState,
        addressInput.error,
        addressInput.isValid,
    ) { current, address, resolve, addressError, isValid ->
        val session = current.session
        val addresses = session.addresses.map { it.toPrimitives() }
        ContactEditorUIState(
            isEdit = current.isEdit,
            name = session.name,
            description = session.description,
            avatar = session.avatarImage().listItemImage(current.emojiBackground),
            hasAvatar = session.avatar !is GemContactAvatarChoice.Empty,
            addresses = addresses,
            addressRows = addresses.rows(service),
            addAddressListItem = addAddressListItem(context),
            page = current.page,
            isSaving = session.isSaving,
            saved = current.saved,
            errorText = current.errorText,
            isSaveEnabled = session.canSave(),
            addressInput = current.form?.let { form ->
                ContactAddressInput(
                    editingId = form.editingId,
                    chain = form.chain,
                    memo = form.memo,
                    address = address,
                    nameResolveIndicator = resolve.indicator(),
                    isAddressValid = isValid,
                    addressError = addressError?.string(context).orEmpty(),
                    showsMemo = GemContactAddressField.MEMO in form.fields,
                )
            },
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ContactEditorUIState(isEdit = mode is Mode.Edit))

    init {
        when (val mode = mode) {
            is Mode.Edit -> viewModelScope.launch(ioDispatcher) {
                val data = getContacts.getContact(mode.contactId) ?: return@launch
                updateSession { service.newSession(data.contact.toGem(), data.addresses.map { address -> address.toGem() }) }
            }

            Mode.Add -> Unit
        }
    }

    private fun updateSession(transform: (GemContactSession) -> GemContactSession) = state.update { it.copy(session = transform(it.session)) }

    fun back() {
        when (state.value.page) {
            ContactEditorPage.Form -> Unit
            ContactEditorPage.Address -> cancelAddress()
            ContactEditorPage.SelectChain -> cancelSelectChain()
            ContactEditorPage.Avatar -> cancelAvatar()
        }
    }

    fun setName(value: String) = updateSession { it.onNameChanged(value) }

    fun setDescription(value: String) = updateSession { it.onDescriptionChanged(value) }

    fun selectAvatar() = state.update { it.copy(page = ContactEditorPage.Avatar) }

    fun cancelAvatar() = state.update { it.copy(page = ContactEditorPage.Form) }

    fun setAvatar(emoji: String, backgroundColor: Int) = state.update {
        it.copy(
            session = it.session.onAvatarChanged(GemContactAvatarChoice.Emoji(emoji)),
            emojiBackground = backgroundColor,
            page = ContactEditorPage.Form,
        )
    }

    fun removeAvatar() = updateSession { it.onAvatarChanged(GemContactAvatarChoice.Empty) }

    fun deleteAddress(address: ContactAddress) = updateSession { it.onAddressDeleted(address.id) }

    fun addAddress() {
        val form = ContactAddressForm(chain = service.defaultChain().requireChain())
        addressInput.reset()
        addressInput.setChain(form.chain)
        state.update { it.copy(page = ContactEditorPage.Address, form = form) }
    }

    fun editAddress(address: ContactAddress) {
        addressInput.reset()
        addressInput.setChain(address.chain)
        addressInput.onTextChange(address.address)
        state.update {
            it.copy(
                page = ContactEditorPage.Address,
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
        state.update { it.copy(page = ContactEditorPage.Form) }
    }

    fun setAddress(value: String) = addressInput.onTextChange(value)

    fun setMemo(value: String) = updateInput { it.copy(memo = value) }

    fun scanAddress(data: String) = setScannedAddress(data)

    fun pasteAddress(data: String) = setScannedAddress(data)

    private fun setScannedAddress(data: String) {
        val scan = service.scannedAddress(data)
        addressInput.setScannedAddress(scan.address)
        updateInput { it.copy(memo = scan.memo ?: it.memo) }
    }

    fun selectChain() = state.update { it.copy(page = ContactEditorPage.SelectChain) }

    fun cancelSelectChain() = state.update { it.copy(page = ContactEditorPage.Address) }

    fun setChain(chain: Chain) {
        addressInput.setChain(chain)
        state.update {
            it.copy(page = ContactEditorPage.Address, form = it.form?.copy(chain = chain, memo = "", fields = contactAddressFields(chain.string)))
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
                session = current.session.onAddressSaved(
                    GemContactAddressInput(
                        contactId = current.session.id,
                        chain = input.chain.string,
                        address = address,
                        memo = input.memo,
                        replacingId = input.editingId,
                    ),
                ),
                page = ContactEditorPage.Form,
            )
        }
    }

    fun save() {
        val current = state.value
        if (!current.session.canSave()) return
        updateSession { it.onSaving(true) }

        viewModelScope.launch(ioDispatcher) {
            val input = current.session.input(
                when (val avatar = current.session.avatar) {
                    GemContactAvatarChoice.Empty -> GemContactAvatar.Empty
                    is GemContactAvatarChoice.Image -> GemContactAvatar.Image(avatar.imageUrl)
                    is GemContactAvatarChoice.Emoji -> GemContactAvatar.Rendered(EmojiAvatarRenderer.render(context, avatar.emoji, current.emojiBackground))
                },
            )
            runCatchingCancellable { service.saveContact(input) }
                .onSuccess { state.update { it.copy(saved = true) } }
                .onFailure { error ->
                    state.update { it.copy(session = it.session.onSaving(false), errorText = error.errorText().text(context)) }
                }
        }
    }

    fun clearError() = state.update { it.copy(errorText = null) }
}
