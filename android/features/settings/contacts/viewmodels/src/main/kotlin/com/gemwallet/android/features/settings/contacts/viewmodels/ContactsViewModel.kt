package com.gemwallet.android.features.settings.contacts.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemContactServiceInterface
import android.util.Log
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

@HiltViewModel
class ContactsViewModel @Inject constructor(
    getContacts: GetContacts,
    private val service: GemContactServiceInterface,
) : ViewModel() {

    val contacts: StateFlow<List<ContactData>> = getContacts.getContacts()
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun deleteContact(contact: Contact) {
        viewModelScope.launch(Dispatchers.IO) {
            runCatchingCancellable { service.deleteContact(contact.toJson()) }
                .onFailure { Log.e(TAG, "deleting contact ${contact.id} failed", it) }
        }
    }

    private companion object {
        const val TAG = "Contacts"
    }
}
