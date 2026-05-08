package com.gemwallet.android.data.service.store.database.entities

import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
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
            entity = DbAsset::class,
            parentColumns = ["id"],
            childColumns = ["chain"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
        ForeignKey(
            entity = DbWallet::class,
            parentColumns = ["id"],
            childColumns = ["walletId"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
    ],
    indices = [Index("chain"), Index("walletId")],
)
data class DbAddress(
    val chain: Chain,
    val address: String,
    val walletId: String?,
    val name: String,
    val type: AddressType,
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

fun List<AddressName>.toRecord(): List<DbAddress> = map { it.toRecord() }

fun Wallet.toAddressRecords(): List<DbAddress> = accounts.map { account ->
    DbAddress(
        chain = account.chain,
        address = account.address,
        walletId = id,
        name = name,
        type = AddressType.InternalWallet,
        status = VerificationStatus.Verified,
    )
}
