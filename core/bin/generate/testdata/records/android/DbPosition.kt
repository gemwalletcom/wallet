package com.gemwallet.android.data.services.store.database.entities

@Entity(
    tableName = "positions",
    primaryKeys = ["id", "wallet_id"],
)
data class DbPosition(
    val id: String,
    @ColumnInfo("wallet_id") val walletId: String,
    val leverage: Int,
    val entryPrice: Double? = null, // written before the price was known
    val takeProfitPrice: Double? = null,
    val takeProfitOrderId: String? = null,
    @ColumnInfo(name = "preview_image_url", defaultValue = "") val previewImageUrl: String = "",
    val tags: List<String>,
    val state: ConnectionState,
    val updatedAt: Long,
    val isPinned: Boolean = false,
)
