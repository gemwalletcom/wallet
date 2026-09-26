package com.gemwallet.android.data.services.store.database.entities

import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.VerificationStatus

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
data class DbAddress(val chain: Chain, val address: String, val walletId: String? = null, val name: String, val type: AddressType, val status: VerificationStatus, val imageUrl: String? = null)

data class AddressNameUpdate(val address: DbAddress, val replacesTypes: List<AddressType>)
