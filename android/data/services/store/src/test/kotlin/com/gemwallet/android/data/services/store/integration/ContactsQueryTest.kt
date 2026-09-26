package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbContact
import com.gemwallet.android.data.services.store.database.entities.DbContactAddress
import com.gemwallet.android.data.services.store.queries.ContactQuery
import com.gemwallet.android.data.services.store.queries.ContactsQuery
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class ContactsQueryTest {
    private lateinit var database: GemDatabase

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        val contacts = database.contactsDao()
        contacts.addContact(
            DbContact(id = "bob", name = "Bob", createdAt = 1, updatedAt = 1),
            listOf(
                DbContactAddress(id = "bob-eth", contactId = "bob", address = "0xbob", chain = Chain.Ethereum),
                DbContactAddress(id = "bob-btc", contactId = "bob", address = "bc1bob", chain = Chain.Bitcoin),
            ),
        )
        contacts.addContact(
            DbContact(id = "alice", name = "Alice", createdAt = 2, updatedAt = 2),
            listOf(DbContactAddress(id = "alice-eth", contactId = "alice", address = "0xalice", chain = Chain.Ethereum, memo = "memo")),
        )
        contacts.addContact(DbContact(id = "carol", name = "Carol", createdAt = 3, updatedAt = 3), emptyList())
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun listsEveryContactByNameWithAllItsAddresses() = runBlocking(Dispatchers.IO) {
        val contacts = ContactsQuery(database.contactsDao())().first()

        assertEquals(listOf("Alice", "Bob", "Carol"), contacts.map { it.contact.name })
        assertEquals(listOf(setOf("alice-eth"), setOf("bob-eth", "bob-btc"), emptySet()), contacts.map { data -> data.addresses.map { it.id }.toSet() })
    }

    @Test
    fun readsOneContactWithItsAddresses() = runBlocking(Dispatchers.IO) {
        val request = ContactQuery(database.contactsDao())

        assertEquals(setOf("bob-eth", "bob-btc"), request("bob")?.addresses?.map { it.id }?.toSet())
        assertNull(request("dave"))
    }
}
