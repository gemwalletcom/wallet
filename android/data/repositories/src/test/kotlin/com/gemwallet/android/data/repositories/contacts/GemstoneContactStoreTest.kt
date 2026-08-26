package com.gemwallet.android.data.repositories.contacts

import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.ContactsDao
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockContact
import com.gemwallet.android.testkit.mockContactAddress
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Test

class GemstoneContactStoreTest {

    private val contactsDao = mockk<ContactsDao>(relaxed = true)
    private val addressesDao = mockk<AddressesDao>(relaxed = true)
    private val store = GemstoneContactStore(contactsDao, addressesDao)

    @Test
    fun updateContact_removesDroppedAddressesFromAddressBook() = runTest {
        coEvery { contactsDao.getAddresses("contact-1") } returns listOf(
            mockContactAddress("a1", Chain.Bitcoin).toRecord(),
            mockContactAddress("a2", Chain.Ethereum).toRecord(),
        )

        store.updateContact(
            contact = mockContact("contact-1").toJson(),
            addresses = listOf(mockContactAddress("a1", Chain.Bitcoin).toJson()),
            deleteAddressIds = listOf("a2"),
        )

        coVerify { contactsDao.updateContact(any(), listOf("a2"), any()) }
        coVerify { addressesDao.delete(Chain.Ethereum, "address-a2", AddressType.Contact) }
        coVerify(exactly = 0) { addressesDao.delete(Chain.Bitcoin, "address-a1", AddressType.Contact) }
    }

    @Test
    fun deleteContact_removesContactAddressBookEntries() = runTest {
        coEvery { contactsDao.getAddresses("contact-1") } returns listOf(
            mockContactAddress("a1", Chain.Bitcoin).toRecord(),
            mockContactAddress("a2", Chain.Ethereum).toRecord(),
        )

        store.deleteContact("contact-1")

        coVerify { addressesDao.delete(Chain.Bitcoin, "address-a1", AddressType.Contact) }
        coVerify { addressesDao.delete(Chain.Ethereum, "address-a2", AddressType.Contact) }
        coVerify { contactsDao.deleteContact("contact-1") }
    }
}
