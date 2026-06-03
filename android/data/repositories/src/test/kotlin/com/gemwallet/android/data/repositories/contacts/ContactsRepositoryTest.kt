package com.gemwallet.android.data.repositories.contacts

import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.ContactsDao
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactAddress
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import io.mockk.slot
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class ContactsRepositoryTest {

    private val contactsDao = mockk<ContactsDao>(relaxed = true)
    private val addressesDao = mockk<AddressesDao>(relaxed = true)
    private val repository = ContactsRepository(contactsDao, addressesDao)

    @Test
    fun updateContact_deletesAddressesMissingFromTheNewSet() = runTest {
        coEvery { contactsDao.getAddresses("contact-1") } returns listOf(
            address("a1", Chain.Bitcoin).toRecord(),
            address("a2", Chain.Bitcoin).toRecord(),
            address("a3", Chain.Ethereum).toRecord(),
        )
        val deleteIds = slot<List<String>>()

        repository.updateContact(
            contact = contact("contact-1"),
            addresses = listOf(address("a1", Chain.Bitcoin), address("a4", Chain.Ethereum)),
        )

        coVerify { contactsDao.updateContact(any(), capture(deleteIds), any()) }
        assertEquals(setOf("a2", "a3"), deleteIds.captured.toSet())
    }

    @Test
    fun updateContact_removesDroppedAddressesFromAddressBook() = runTest {
        coEvery { contactsDao.getAddresses("contact-1") } returns listOf(
            address("a1", Chain.Bitcoin).toRecord(),
            address("a2", Chain.Ethereum).toRecord(),
        )

        repository.updateContact(
            contact = contact("contact-1"),
            addresses = listOf(address("a1", Chain.Bitcoin)),
        )

        coVerify { addressesDao.delete(Chain.Ethereum, "address-a2", AddressType.Contact) }
        coVerify(exactly = 0) { addressesDao.delete(Chain.Bitcoin, "address-a1", AddressType.Contact) }
    }

    @Test
    fun deleteContact_removesContactAddressBookEntries() = runTest {
        coEvery { contactsDao.getAddresses("contact-1") } returns listOf(
            address("a1", Chain.Bitcoin).toRecord(),
            address("a2", Chain.Ethereum).toRecord(),
        )

        repository.deleteContact("contact-1")

        coVerify { addressesDao.delete(Chain.Bitcoin, "address-a1", AddressType.Contact) }
        coVerify { addressesDao.delete(Chain.Ethereum, "address-a2", AddressType.Contact) }
        coVerify { contactsDao.deleteContact("contact-1") }
    }

    private fun contact(id: String) = Contact(
        id = id,
        name = "John",
        description = null,
        createdAt = 0L,
        updatedAt = 1L,
    )

    private fun address(id: String, chain: Chain) = ContactAddress(
        id = id,
        contactId = "contact-1",
        address = "address-$id",
        chain = chain,
        memo = null,
    )
}
