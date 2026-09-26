package com.gemwallet.android.features.contacts.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.data.services.store.queries.ContactsQuery
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.contacts.viewmodels.models.ContactRowUIModel
import com.gemwallet.android.features.contacts.viewmodels.models.listItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.navigation.ContactAddressDraft
import com.gemwallet.android.ui.models.navigation.optionalContactAddressDraft
import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactData
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemContactAddressInput
import uniffi.gemstone.GemContactRow
import uniffi.gemstone.GemContactServiceInterface
import uniffi.gemstone.contactRows
import javax.inject.Inject

@HiltViewModel
class ContactsViewModel @Inject constructor(
    contactsQuery: ContactsQuery,
    private val service: GemContactServiceInterface,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val contacts: StateFlow<List<ContactRowUIModel>> = contactsQuery()
        .map { contacts -> contacts.zip(contactRows(contacts.map { it.contact.toGem() }), ::listItem) }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private fun listItem(contact: ContactData, row: GemContactRow): ContactRowUIModel = ContactRowUIModel(
        contact = contact,
        model = ListItemModel(
            title = row.title,
            titleExtra = row.subtitle,
            titleExtraLineLimit = 1,
            image = row.avatar.listItemImage(emojiBackground = 0),
        ),
    )

    private val errorState = MutableStateFlow<String?>(null)
    val errorText: StateFlow<String?> = errorState.asStateFlow()

    val draft: ContactAddressDraft? = savedStateHandle.optionalContactAddressDraft()

    fun addAddress(contact: ContactData, onAdded: () -> Unit) {
        val draft = draft ?: return
        viewModelScope.launch(ioDispatcher) {
            val input = GemContactAddressInput(contactId = contact.contact.id, chain = draft.chain.string, address = draft.address, memo = draft.memo, replacingId = null)
            runCatchingCancellable { service.updateContact(contact.contact.toGem(), input.addAddress(contact.addresses.map { it.toGem() })) }
                .onSuccess { withContext(Dispatchers.Main) { onAdded() } }
                .onFailure { errorState.value = it.errorText().text(context) }
        }
    }

    fun deleteContact(contact: Contact) {
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { service.deleteContact(contact.toGem()) }
                .onFailure { errorState.value = it.errorText().text(context) }
        }
    }

    fun clearError() = errorState.update { null }
}
