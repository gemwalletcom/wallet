package com.gemwallet.android.features.settings.contacts.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.contacts.cases.GetContacts
import com.gemwallet.android.data.services.gemstone.di.IoDispatcher
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactAvatarState
import com.gemwallet.android.features.settings.contacts.viewmodels.models.image
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactData
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemContactServiceInterface
import uniffi.gemstone.contactRow

@HiltViewModel
class ContactsViewModel @Inject constructor(
    getContacts: GetContacts,
    private val service: GemContactServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val contacts: StateFlow<List<ContactData>> = getContacts.getContacts()
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun listItem(contact: ContactData): ListItemModel {
        val row = contactRow(contact.contact.toGem())
        return ListItemModel(
            title = row.title,
            titleExtra = row.subtitle,
            image = ContactAvatarState.from(contact.contact.imageUrl).image(row.initials),
        )
    }

    private val errorState = MutableStateFlow<String?>(null)
    val errorText: StateFlow<String?> = errorState.asStateFlow()

    fun deleteContact(contact: Contact) {
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { service.deleteContact(contact.toGem()) }
                .onFailure { errorState.value = it.errorText().text(context) }
        }
    }

    fun clearError() = errorState.update { null }

}
