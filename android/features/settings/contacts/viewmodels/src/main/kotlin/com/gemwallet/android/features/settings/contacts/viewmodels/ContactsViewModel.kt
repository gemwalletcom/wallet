package com.gemwallet.android.features.settings.contacts.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.cases.contacts.DeleteContact
import com.gemwallet.android.cases.contacts.GetContacts
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
    private val deleteContactCase: DeleteContact,
) : ViewModel() {

    val contacts: StateFlow<List<ContactData>> = getContacts.getContacts()
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun deleteContact(id: String) {
        viewModelScope.launch(Dispatchers.IO) {
            deleteContactCase.deleteContact(id)
        }
    }
}
