package com.gemwallet.android.data.service.store.database.entities

import androidx.room.Embedded
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey
import androidx.room.Relation
import com.gemwallet.android.cases.contacts.ContactRecipient
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactAddress
import com.wallet.core.primitives.ContactData

@Entity(tableName = "contacts")
data class DbContact(
    @PrimaryKey val id: String,
    val name: String,
    val description: String? = null,
    val createdAt: Long,
    val updatedAt: Long,
)

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
data class DbContactAddress(
    @PrimaryKey val id: String,
    val contactId: String,
    val address: String,
    val chain: Chain,
    val memo: String? = null,
)

data class DbContactWithAddresses(
    @Embedded val contact: DbContact,
    @Relation(parentColumn = "id", entityColumn = "contactId")
    val addresses: List<DbContactAddress>,
)

data class DbRecipientContact(
    val name: String,
    val address: String,
    val chain: Chain,
    val memo: String?,
)

fun Contact.toRecord(): DbContact = DbContact(
    id = id,
    name = name,
    description = description,
    createdAt = createdAt,
    updatedAt = updatedAt,
)

fun ContactAddress.toRecord(): DbContactAddress = DbContactAddress(
    id = id,
    contactId = contactId,
    address = address,
    chain = chain,
    memo = memo,
)

fun DbContact.toModel(): Contact = Contact(
    id = id,
    name = name,
    description = description,
    createdAt = createdAt,
    updatedAt = updatedAt,
)

fun DbContactAddress.toModel(): ContactAddress = ContactAddress(
    id = id,
    contactId = contactId,
    address = address,
    chain = chain,
    memo = memo,
)

fun DbContactWithAddresses.toModel(): ContactData = ContactData(
    contact = contact.toModel(),
    addresses = addresses.map { it.toModel() },
)

fun DbRecipientContact.toModel(): ContactRecipient = ContactRecipient(
    name = name,
    address = address,
    chain = chain,
    memo = memo,
)
