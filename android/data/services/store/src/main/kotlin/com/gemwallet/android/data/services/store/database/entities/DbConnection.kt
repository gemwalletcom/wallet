package com.gemwallet.android.data.services.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletConnection
import com.wallet.core.primitives.WalletConnectionState

@Entity(
    tableName = "wallets_connections",
    foreignKeys = [
        ForeignKey(DbWallet::class, ["id"], ["wallet_id"], onDelete = ForeignKey.CASCADE, onUpdate = ForeignKey.CASCADE),
    ],
    indices = [Index("wallet_id")],
)
data class DbConnection(
    @PrimaryKey val id: String,
    @ColumnInfo("wallet_id") val walletId: String,
    @ColumnInfo("session_id") val sessionId: String,
    val state: WalletConnectionState,
    val chains: List<Chain>,
    @ColumnInfo("created_at") val createdAt: Long,
    @ColumnInfo("expire_at") val expireAt: Long,
    @ColumnInfo("app_name") val appName: String,
    @ColumnInfo("app_description") val appDescription: String,
    @ColumnInfo("app_url") val appUrl: String,
    @ColumnInfo("app_icon") val appIcon: String,
    @ColumnInfo("redirect_native") val redirectNative: String? = null,
    @ColumnInfo("redirect_universal") val redirectUniversal: String? = null,
)

fun DbConnection.toDTO(wallet: Wallet): WalletConnection = WalletConnection(wallet = wallet, session = toWalletConnectionSession())

fun DbConnection.toDTO(wallets: List<Wallet>): WalletConnection? = wallets.firstOrNull { it.id.id == walletId }?.let { toDTO(it) }
