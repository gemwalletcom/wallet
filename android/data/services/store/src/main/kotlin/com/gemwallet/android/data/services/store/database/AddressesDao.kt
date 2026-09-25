package com.gemwallet.android.data.services.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Transaction
import com.gemwallet.android.data.services.store.database.entities.AddressNameUpdate
import com.gemwallet.android.data.services.store.database.entities.DbAddress
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.VerificationStatus

@Dao
interface AddressesDao {

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(addresses: List<DbAddress>)

    @Transaction
    suspend fun updateNames(updates: List<AddressNameUpdate>) {
        updates.forEach {
            val address = it.address
            updateAddressName(address.chain, address.address, address.name, address.type, address.status, address.imageUrl, it.replacesTypes)
        }
        insertIgnore(updates.map { it.address })
    }

    @Query(
        "UPDATE addresses SET name = :name, type = :type, status = :status, imageUrl = :imageUrl " +
            "WHERE chain = :chain AND address = :address AND type IN (:replacesTypes)",
    )
    suspend fun updateAddressName(chain: Chain, address: String, name: String, type: AddressType, status: VerificationStatus, imageUrl: String?, replacesTypes: List<AddressType>)

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
