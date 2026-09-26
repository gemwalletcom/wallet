package com.gemwallet.android.data.services.store.database.entities

import androidx.room.Embedded
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey
import androidx.room.Relation
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ContactData

@Entity(tableName = "contacts")
data class DbContact(@PrimaryKey val id: String, val name: String, val description: String? = null, val imageUrl: String? = null, val createdAt: Long, val updatedAt: Long)

@Entity(
    tableName = "contacts_addresses",
    foreignKeys = [
        ForeignKey(
            entity = DbContact::class,
            parentColumns = ["id"],
            childColumns = ["contactId"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
    ],
    indices = [Index("contactId")],
)
data class DbContactAddress(@PrimaryKey val id: String, val contactId: String, val address: String, val chain: Chain, val memo: String? = null)

data class DbContactWithAddresses(
    @Embedded val contact: DbContact,
    @Relation(parentColumn = "id", entityColumn = "contactId")
    val addresses: List<DbContactAddress>,
)

fun DbContactWithAddresses.toModel(): ContactData = ContactData(
    contact = contact.toContact(),
    addresses = addresses.map { it.toContactAddress() },
)
