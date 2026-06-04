package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Query
import androidx.room.Transaction
import androidx.room.Upsert
import com.gemwallet.android.data.service.store.database.entities.DbDelegationBase
import com.gemwallet.android.data.service.store.database.entities.DbDelegationData
import com.gemwallet.android.data.service.store.database.entities.DbDelegationValidator
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

@Dao
interface StakeDao {
    @Upsert
    suspend fun upsertValidators(validators: List<DbDelegationValidator>)

    @Upsert
    suspend fun upsertDelegations(delegations: List<DbDelegationBase>)

    @Transaction
    suspend fun updateAndDeleteDelegations(
        walletId: WalletId,
        delegations: List<DbDelegationBase>,
        deleteIds: List<String>,
    ) {
        if (delegations.isNotEmpty()) {
            upsertDelegations(delegations)
        }
        if (deleteIds.isNotEmpty()) {
            deleteDelegations(walletId, deleteIds)
        }
    }

    @Query("DELETE FROM stake_delegations WHERE walletId=:walletId AND id IN (:ids)")
    suspend fun deleteDelegations(walletId: WalletId, ids: List<String>)

    @Query("SELECT id FROM stake_delegations WHERE walletId=:walletId AND assetId=:assetId")
    suspend fun getDelegationIds(walletId: WalletId, assetId: AssetId): List<String>

    @Query(
        "SELECT * FROM stake_validators WHERE assetId=:assetId AND providerType=:providerType " +
            "ORDER BY apr DESC"
    )
    fun getValidators(assetId: AssetId, providerType: StakeProviderType): Flow<List<DbDelegationValidator>>

    @Query("SELECT * FROM stake_validators WHERE assetId=:assetId AND validatorId=:validatorId LIMIT 1")
    suspend fun getValidator(assetId: AssetId, validatorId: String): DbDelegationValidator?

    @Transaction
    @Query("SELECT * FROM stake_delegations WHERE walletId=:walletId AND assetId=:assetId")
    fun getDelegations(walletId: WalletId, assetId: AssetId): Flow<List<DbDelegationData>>

    @Transaction
    @Query(
        "SELECT base.* FROM stake_delegations as base " +
            "INNER JOIN stake_validators as validator ON base.validatorId=validator.id " +
            "WHERE base.delegationId=:delegationId AND validator.validatorId=:validatorId LIMIT 1"
    )
    fun getDelegation(validatorId: String, delegationId: String): Flow<DbDelegationData?>
}
