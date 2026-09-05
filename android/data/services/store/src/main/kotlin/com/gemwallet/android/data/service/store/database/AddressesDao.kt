package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Transaction
import com.gemwallet.android.data.service.store.database.entities.DbAddress
import com.gemwallet.android.data.service.store.database.entities.isLocal
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.VerificationStatus

@Dao
interface AddressesDao {

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(addresses: List<DbAddress>)

    @Transaction
    suspend fun updateNames(addresses: List<DbAddress>) {
        addresses.forEach {
            updateAddressName(it.chain, it.address, it.name, it.type, it.status, it.imageUrl)
        }
        insertIgnore(addresses)
    }

    @Query(
        "UPDATE addresses SET name = :name, type = :type, status = :status, imageUrl = :imageUrl " +
            "WHERE chain = :chain AND address = :address AND type NOT IN (:reservedTypes)"
    )
    suspend fun updateAddressName(
        chain: Chain,
        address: String,
        name: String,
        type: AddressType,
        status: VerificationStatus,
        imageUrl: String?,
        reservedTypes: List<AddressType> = AddressType.entries.filter { it.isLocal && it != type },
    )

    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun insertIgnore(addresses: List<DbAddress>)

    @Query("SELECT * FROM addresses WHERE chain = :chain AND address = :address LIMIT 1")
    suspend fun get(chain: Chain, address: String): DbAddress?

    @Query("DELETE FROM addresses WHERE chain = :chain AND address = :address AND type = :type")
    suspend fun delete(chain: Chain, address: String, type: AddressType)

    @Transaction
    suspend fun deleteNames(addresses: List<DbAddress>) {
        addresses.forEach { delete(it.chain, it.address, it.type) }
    }
}
