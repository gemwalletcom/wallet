package com.gemwallet.android.data.repositories.contacts

import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.ContactsDao
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.testkit.mockContact
import com.gemwallet.android.testkit.mockContactAddress
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Chain
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
            mockContactAddress("a1", Chain.Bitcoin).toRecord(),
            mockContactAddress("a2", Chain.Bitcoin).toRecord(),
            mockContactAddress("a3", Chain.Ethereum).toRecord(),
        )
        val deleteIds = slot<List<String>>()

        repository.updateContact(
            contact = mockContact("contact-1"),
            addresses = listOf(mockContactAddress("a1", Chain.Bitcoin), mockContactAddress("a4", Chain.Ethereum)),
        )

        coVerify { contactsDao.updateContact(any(), capture(deleteIds), any()) }
        assertEquals(setOf("a2", "a3"), deleteIds.captured.toSet())
    }

    @Test
    fun updateContact_removesDroppedAddressesFromAddressBook() = runTest {
        coEvery { contactsDao.getAddresses("contact-1") } returns listOf(
            mockContactAddress("a1", Chain.Bitcoin).toRecord(),
            mockContactAddress("a2", Chain.Ethereum).toRecord(),
        )

        repository.updateContact(
            contact = mockContact("contact-1"),
            addresses = listOf(mockContactAddress("a1", Chain.Bitcoin)),
        )

        coVerify { addressesDao.delete(Chain.Ethereum, "address-a2", AddressType.Contact) }
        coVerify(exactly = 0) { addressesDao.delete(Chain.Bitcoin, "address-a1", AddressType.Contact) }
    }

    @Test
    fun deleteContact_removesContactAddressBookEntries() = runTest {
        coEvery { contactsDao.getAddresses("contact-1") } returns listOf(
            mockContactAddress("a1", Chain.Bitcoin).toRecord(),
            mockContactAddress("a2", Chain.Ethereum).toRecord(),
        )

        repository.deleteContact("contact-1")

        coVerify { addressesDao.delete(Chain.Bitcoin, "address-a1", AddressType.Contact) }
        coVerify { addressesDao.delete(Chain.Ethereum, "address-a2", AddressType.Contact) }
        coVerify { contactsDao.deleteContact("contact-1") }
    }
}
