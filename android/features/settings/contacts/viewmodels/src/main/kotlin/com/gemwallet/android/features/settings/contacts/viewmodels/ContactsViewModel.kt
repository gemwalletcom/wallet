package com.gemwallet.android.features.settings.contacts.viewmodels

import com.gemwallet.android.ext.toGem
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.ext.runCatchingCancellable
import uniffi.gemstone.GemContactServiceInterface
import com.gemwallet.android.application.contacts.cases.GetContacts
import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactData
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import com.gemwallet.android.ext.serviceMessage

@HiltViewModel
class ContactsViewModel @Inject constructor(
    getContacts: GetContacts,
    private val service: GemContactServiceInterface,
) : ViewModel() {

    val contacts: StateFlow<List<ContactData>> = getContacts.getContacts()
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

    fun deleteContact(contact: Contact) {
        viewModelScope.launch(Dispatchers.IO) {
            runCatchingCancellable { service.deleteContact(contact.toGem()) }
                .onFailure { errorState.value = it.serviceMessage() }
        }
    }

    fun clearError() = errorState.update { null }

}
