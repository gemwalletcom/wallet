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
import kotlinx.coroutines.flow.Flow

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
            "WHERE chain = :chain AND address = :address AND type NOT IN (:localTypes)"
    )
    suspend fun updateAddressName(
        chain: Chain,
        address: String,
        name: String,
        type: AddressType,
        status: VerificationStatus,
        imageUrl: String?,
        localTypes: List<AddressType> = AddressType.entries.filter { it.isLocal },
    )

    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun insertIgnore(addresses: List<DbAddress>)

    @Query("SELECT * FROM addresses WHERE chain = :chain AND address = :address LIMIT 1")
    fun getFlow(chain: Chain, address: String): Flow<DbAddress?>

    @Query("SELECT * FROM addresses WHERE chain = :chain AND address = :address LIMIT 1")
    suspend fun get(chain: Chain, address: String): DbAddress?

    @Query("SELECT * FROM addresses WHERE chain = :chain AND address = :address LIMIT 1")
    fun getNow(chain: Chain, address: String): DbAddress?

    @Query("DELETE FROM addresses WHERE chain = :chain AND address = :address AND type = :type")
    suspend fun delete(chain: Chain, address: String, type: AddressType)

    @Transaction
    suspend fun deleteNames(addresses: List<DbAddress>) {
        addresses.forEach { delete(it.chain, it.address, it.type) }
    }

    @Query("UPDATE addresses SET name = :name WHERE walletId = :walletId")
    suspend fun updateName(walletId: String, name: String)
}
