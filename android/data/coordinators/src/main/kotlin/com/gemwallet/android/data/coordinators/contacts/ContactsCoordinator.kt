package com.gemwallet.android.data.coordinators.contacts

import com.gemwallet.android.application.contacts.cases.AddContactAddress
import com.gemwallet.android.application.contacts.cases.DeleteContact
import com.gemwallet.android.application.contacts.cases.SaveContact
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactAddress
import uniffi.gemstone.GemContactAddressInput
import uniffi.gemstone.GemContactAvatar
import uniffi.gemstone.GemContactInput
import uniffi.gemstone.GemContactService

class ContactsCoordinator(
    private val contactService: GemContactService,
) : SaveContact, AddContactAddress, DeleteContact {

    override suspend fun saveContact(
        id: String,
        existing: Contact?,
        name: String,
        description: String,
        avatar: GemContactAvatar,
        addresses: List<ContactAddress>,
    ): Contact = contactService.saveContact(
        GemContactInput(
            id = id,
            existing = existing?.toJson(),
            name = name,
            description = description,
            avatar = avatar,
            addresses = addresses.map { it.toJson() },
        )
    ).decodeJson<Contact>()

    override suspend fun saveAddresses(contact: Contact, addresses: List<ContactAddress>) =
        contactService.updateContact(contact.toJson(), addresses.map { it.toJson() })

    override fun addAddress(
        addresses: List<ContactAddress>,
        contactId: String,
        chain: Chain,
        address: String,
        memo: String?,
        replacingId: String?,
    ): List<ContactAddress> = GemContactAddressInput(
        contactId = contactId,
        chain = chain.string,
        address = address,
        memo = memo,
        replacingId = replacingId,
    )
        .addAddress(addresses.map { it.toJson() })
        .map { it.decodeJson<ContactAddress>() }

    override fun defaultChain(): Chain = contactService.defaultChain().toChain() ?: Chain.Bitcoin

    override suspend fun deleteContact(contact: Contact) = contactService.deleteContact(contact.toJson())
}
