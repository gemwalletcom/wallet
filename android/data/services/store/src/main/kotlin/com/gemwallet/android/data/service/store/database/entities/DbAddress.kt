package com.gemwallet.android.data.service.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.VerificationStatus
import com.wallet.core.primitives.Wallet

@Entity(
    tableName = "addresses",
    primaryKeys = ["chain", "address"],
    foreignKeys = [
        ForeignKey(
            entity = DbWallet::class,
            parentColumns = ["id"],
            childColumns = ["wallet_id"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
    ],
    indices = [Index("wallet_id")],
)
data class DbAddress(
    val chain: Chain,
    val address: String,
    @ColumnInfo(name = "wallet_id") val walletId: String?,
    val name: String,
    val type: AddressType?,
    val status: VerificationStatus,
)

fun AddressName.toRecord(): DbAddress = DbAddress(
    chain = chain,
    address = address,
    walletId = null,
    name = name,
    type = type,
    status = status,
)

fun DbAddress.toDTO(): AddressName = AddressName(
    chain = chain,
    address = address,
    name = name,
    type = type,
    status = status,
)

fun Account.toAddressRecord(wallet: Wallet): DbAddress = DbAddress(
    chain = chain,
    address = address,
    walletId = wallet.id,
    name = wallet.name,
    type = AddressType.InternalWallet,
    status = VerificationStatus.Verified,
)

fun List<AddressName>.toRecord(): List<DbAddress> = map { it.toRecord() }

fun Wallet.toAddressRecords(): List<DbAddress> = accounts.map { it.toAddressRecord(this) }
