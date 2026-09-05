package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.data.service.store.database.ContactsDao
import com.gemwallet.android.data.service.store.database.entities.DbContactAddress
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class ContactStoreTest {

    private val contactsDao = mockk<ContactsDao>(relaxed = true)
    private val store = GemstoneContactStore(contactsDao)

    @Test
    fun getAddresses_returnsContactAddressesForCore() = runTest {
        coEvery { contactsDao.getAddresses("contact-1") } returns listOf(
            DbContactAddress(id = "address-a1", contactId = "contact-1", chain = Chain.Bitcoin, address = "bc1", memo = null),
        )

        val addresses = store.getAddresses("contact-1").map { it.toPrimitives() }

        assertEquals(listOf("address-a1"), addresses.map { it.id })
        assertEquals(Chain.Bitcoin, addresses.single().chain)
    }
}
