package com.gemwallet.android.data.services.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Embedded
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.Relation
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletAddressItem

@Entity(
    tableName = "accounts",
    primaryKeys = ["wallet_id", "chain"],
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
            childColumns = ["wallet_id"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
    ],
    indices = [Index("chain")],
)
data class DbAccount(@ColumnInfo(name = "wallet_id") val walletId: String, @ColumnInfo(name = "derivation_path") val derivationPath: String, val address: String, val chain: Chain, val extendedPublicKey: String?)

data class DbWalletAddress(@Embedded val account: DbAccount, @Relation(parentColumn = "wallet_id", entityColumn = "id") val wallet: DbWallet)

fun DbWalletAddress.toWalletAddressItem(): WalletAddressItem = WalletAddressItem(wallet = wallet.toWalletListItem(), address = account.address)
