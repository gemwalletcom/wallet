package com.gemwallet.android.data.service.store.database.entities

import androidx.room.Embedded
import androidx.room.Relation
import com.gemwallet.android.ext.toAssetId
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationBase

data class DbDelegationData(
    @Embedded val base: DbDelegationBase,
    @Relation(parentColumn = "validatorId", entityColumn = "id")
    val validator: DbDelegationValidator,
)

fun DbDelegationData.toModel(): Delegation? {
    val asset = base.assetId.toAssetId() ?: return null
    val validatorDTO = validator.toDTO() ?: return null
    return Delegation(
        validator = validatorDTO,
        base = DelegationBase(
            assetId = asset,
            validatorId = validatorDTO.id,
            delegationId = base.delegationId,
            state = base.state,
            balance = base.balance,
            completionDate = base.completionDate,
            rewards = base.rewards,
            shares = base.shares,
        ),
        price = null,
    )
}
