package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Query
import androidx.room.Transaction
import androidx.room.Upsert
import com.gemwallet.android.data.service.store.database.entities.DbDelegationBase
import com.gemwallet.android.data.service.store.database.entities.DbDelegationData
import com.gemwallet.android.data.service.store.database.entities.DbDelegationValidator
import com.wallet.core.primitives.StakeProviderType
import kotlinx.coroutines.flow.Flow

@Dao
interface StakeDao {
    @Upsert
    suspend fun upsertValidators(validators: List<DbDelegationValidator>)

    @Upsert
    suspend fun upsertDelegations(delegations: List<DbDelegationBase>)

    @Query("DELETE FROM stake_delegations WHERE walletId=:walletId AND id IN (:ids)")
    suspend fun deleteDelegations(walletId: String, ids: List<String>)

    @Query(
        "SELECT * FROM stake_validators WHERE assetId=:assetId AND providerType=:providerType " +
            "ORDER BY apr DESC"
    )
    fun getValidators(assetId: String, providerType: StakeProviderType): Flow<List<DbDelegationValidator>>

    @Query("SELECT * FROM stake_validators WHERE assetId=:assetId AND validatorId=:validatorId LIMIT 1")
    suspend fun getValidator(assetId: String, validatorId: String): DbDelegationValidator?

    @Transaction
    @Query("SELECT * FROM stake_delegations WHERE walletId=:walletId AND assetId=:assetId")
    fun getDelegations(walletId: String, assetId: String): Flow<List<DbDelegationData>>

    @Transaction
    @Query(
        "SELECT base.* FROM stake_delegations as base " +
            "INNER JOIN stake_validators as validator ON base.validatorId=validator.id " +
            "WHERE base.delegationId=:delegationId AND validator.validatorId=:validatorId LIMIT 1"
    )
    fun getDelegation(validatorId: String, delegationId: String): Flow<DbDelegationData?>
}
