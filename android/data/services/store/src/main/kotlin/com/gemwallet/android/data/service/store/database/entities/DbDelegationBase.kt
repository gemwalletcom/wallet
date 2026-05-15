package com.gemwallet.android.data.service.store.database.entities

import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.DelegationBase
import com.wallet.core.primitives.DelegationState
import com.wallet.core.primitives.WalletId

@Entity(
    tableName = "stake_delegations",
    primaryKeys = ["walletId", "id"],
    indices = [
        Index("assetId"),
        Index("validatorId"),
    ],
    foreignKeys = [
        ForeignKey(
            entity = DbWallet::class,
            parentColumns = ["id"],
            childColumns = ["walletId"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
        ForeignKey(
            entity = DbAsset::class,
            parentColumns = ["id"],
            childColumns = ["assetId"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
        ForeignKey(
            entity = DbDelegationValidator::class,
            parentColumns = ["id"],
            childColumns = ["validatorId"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
    ],
)
data class DbDelegationBase(
    val id: String,
    val walletId: String,
    val assetId: String,
    val validatorId: String,
    val state: DelegationState,
    val delegationId: String,
    val balance: String,
    val shares: String,
    val rewards: String,
    val completionDate: Long? = null,
)

fun DelegationBase.toRecord(walletId: WalletId): DbDelegationBase {
    return DbDelegationBase(
        id = delegationRecordId(assetId.toIdentifier(), validatorId, state, delegationId),
        walletId = walletId.id,
        assetId = assetId.toIdentifier(),
        validatorId = validatorRecordId(chain = assetId.chain, validatorId = validatorId),
        state = state,
        delegationId = delegationId,
        balance = balance,
        shares = shares,
        rewards = rewards,
        completionDate = completionDate,
    )
}

fun List<DelegationBase>.toRecord(walletId: WalletId) = map { it.toRecord(walletId) }

internal fun delegationRecordId(
    assetId: String,
    validatorId: String,
    state: DelegationState,
    delegationId: String,
): String = "${assetId}_${validatorId}_${state.string}_$delegationId"
