package com.gemwallet.android.data.service.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain

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
data class DbAccount(
    @ColumnInfo(name = "wallet_id") val walletId: String,
    @ColumnInfo(name = "derivation_path") val derivationPath: String,
    val address: String,
    val chain: Chain,
    val extendedPublicKey: String?,
)

fun DbAccount.toDTO(): Account {
    return Account(
        chain = chain,
        address = address,
        extendedPublicKey = extendedPublicKey,
        derivationPath = derivationPath,
    )
}

fun Account.toRecord(walletId: String): DbAccount {
    return DbAccount(
        walletId = walletId,
        derivationPath = derivationPath,
        chain = chain,
        address = address,
        extendedPublicKey = extendedPublicKey,
    )
}

fun List<DbAccount>.toDTO() = map { it.toDTO() }
