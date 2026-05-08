package com.gemwallet.android.data.service.store.database.entities

import androidx.room.Entity
import com.gemwallet.android.ext.hash
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.WalletId

@Entity(tableName = "transactions", primaryKeys = ["id", "walletId"])
data class DbTransaction(
    val id: TransactionId,
    val walletId: WalletId,
    val hash: String,
    val assetId: AssetId,
    val feeAssetId: AssetId,
    val owner: String,
    val recipient: String,
    val contract: String? = null,
    val metadata: String? = null,
    val state: TransactionState,
    val type: TransactionType,
    val blockNumber: String,
    val sequence: String,
    val fee: String, // Atomic value - BigInteger
    val value: String, // Atomic value - BigInteger
    val payload: String? = null,
    val direction: TransactionDirection,
    val createdAt: Long,
    val updatedAt: Long,
)

fun Transaction.toRecord(walletId: WalletId): DbTransaction {
    return DbTransaction(
        id = this.id,
        walletId = walletId,
        hash = this.hash,
        assetId = this.assetId,
        feeAssetId = this.feeAssetId,
        owner = this.from,
        recipient = this.to,
        contract = this.contract,
        type = this.type,
        state = this.state,
        blockNumber = this.blockNumber ?: "",
        sequence = this.sequence ?: "",
        fee = this.fee,
        value = this.value,
        payload = this.memo,
        metadata = this.metadata,
        direction = this.direction,
        updatedAt = System.currentTimeMillis(),
        createdAt = this.createdAt,
    )
}

fun DbTransaction.toDTO(): Transaction {
    return Transaction(
        id = this.id,
        assetId = this.assetId,
        from = this.owner,
        to = this.recipient,
        contract = this.contract,
        type = this.type,
        state = this.state,
        blockNumber = this.blockNumber,
        sequence = this.sequence,
        fee = this.fee,
        feeAssetId = this.feeAssetId,
        value = this.value,
        memo = this.payload,
        direction = this.direction,
        utxoInputs = emptyList(),
        utxoOutputs = emptyList(),
        createdAt = this.createdAt,
        metadata = this.metadata,
    )
}

fun List<DbTransaction>.toDTO() = map { it.toDTO() }

fun List<Transaction>.toRecord(walletId: WalletId) = map { it.toRecord(walletId) }
