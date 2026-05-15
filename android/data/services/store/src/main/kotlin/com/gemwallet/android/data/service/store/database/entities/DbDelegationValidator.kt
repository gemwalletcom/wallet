package com.gemwallet.android.data.service.store.database.entities

import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType

@Entity(
    tableName = "stake_validators",
    indices = [Index("assetId")],
    foreignKeys = [
        ForeignKey(
            entity = DbAsset::class,
            parentColumns = ["id"],
            childColumns = ["assetId"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
    ],
)
data class DbDelegationValidator(
    @PrimaryKey val id: String,
    val assetId: String,
    val validatorId: String,
    val name: String,
    val isActive: Boolean,
    val commission: Double,
    val apr: Double,
    val providerType: StakeProviderType,
)

internal fun validatorRecordId(chain: Chain, validatorId: String): String = "${chain.string}_$validatorId"

fun DbDelegationValidator.toDTO(): DelegationValidator? {
    val chain = assetId.toAssetId()?.chain ?: return null
    return DelegationValidator(
        id = validatorId,
        chain = chain,
        name = name,
        isActive = isActive,
        commission = commission,
        apr = apr,
        providerType = providerType,
    )
}

fun DelegationValidator.toRecord(): DbDelegationValidator {
    return DbDelegationValidator(
        id = validatorRecordId(chain = chain, validatorId = id),
        assetId = AssetId(chain).toIdentifier(),
        validatorId = id,
        name = name,
        isActive = isActive,
        commission = commission,
        apr = apr,
        providerType = providerType,
    )
}

fun List<DbDelegationValidator>.toDTO() = mapNotNull { it.toDTO() }

fun List<DelegationValidator>.toRecord() = map { it.toRecord() }
