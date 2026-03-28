package com.gemwallet.android.data.service.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Entity
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.VerificationStatus

@Entity(tableName = "addresses", primaryKeys = ["chain", "address"])
data class DbAddress(
    val chain: Chain,
    val address: String,
    @ColumnInfo(name = "wallet_id") val walletId: String?,
    val name: String,
    val type: AddressType?,
    val status: VerificationStatus,
)

fun DbAddress.toAddressName(): AddressName {
    return AddressName(
        chain = chain,
        address = address,
        name = name,
        type = type,
        status = status,
    )
}
