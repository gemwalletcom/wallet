package com.gemwallet.android.data.services.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Transaction
import androidx.room.Update
import com.gemwallet.android.data.services.store.database.entities.DbContact
import com.gemwallet.android.data.services.store.database.entities.DbContactAddress
import com.gemwallet.android.data.services.store.database.entities.DbContactWithAddresses
import kotlinx.coroutines.flow.Flow

@Dao
interface ContactsDao {
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertContact(contact: DbContact)

    @Update
    suspend fun updateContactRow(contact: DbContact)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertAddresses(addresses: List<DbContactAddress>)

    @Query("DELETE FROM contacts_addresses WHERE id IN (:ids)")
    suspend fun deleteAddresses(ids: List<String>)

    @Query("SELECT * FROM contacts_addresses WHERE contactId = :contactId")
    suspend fun getAddresses(contactId: String): List<DbContactAddress>

    @Query("DELETE FROM contacts WHERE id = :id")
    suspend fun deleteContact(id: String)

    @Transaction
    @Query("SELECT * FROM contacts ORDER BY name ASC")
    fun getContacts(): Flow<List<DbContactWithAddresses>>

    @Transaction
    @Query("SELECT * FROM contacts WHERE id = :id LIMIT 1")
    suspend fun getContact(id: String): DbContactWithAddresses?

    @Transaction
    suspend fun addContact(contact: DbContact, addresses: List<DbContactAddress>) {
        insertContact(contact)
        if (addresses.isNotEmpty()) {
            insertAddresses(addresses)
        }
    }

    @Transaction
    suspend fun updateContact(contact: DbContact, deleteAddressIds: List<String>, addresses: List<DbContactAddress>) {
        updateContactRow(contact)
        if (deleteAddressIds.isNotEmpty()) {
            deleteAddresses(deleteAddressIds)
        }
        if (addresses.isNotEmpty()) {
            insertAddresses(addresses)
        }
    }
}
