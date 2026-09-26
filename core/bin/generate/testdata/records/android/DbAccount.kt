package com.gemwallet.android.data.services.store.database.entities

@Entity(tableName = "accounts", primaryKeys = ["wallet_id", "chain"])
data class DbAccount(@ColumnInfo(name = "wallet_id") val walletId: String, @ColumnInfo(name = "derivation_path") val derivationPath: String, val address: String, val chain: Chain, val extendedPublicKey: String?)
