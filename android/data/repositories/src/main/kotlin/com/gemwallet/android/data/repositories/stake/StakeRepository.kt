package com.gemwallet.android.data.repositories.stake

import com.gemwallet.android.cases.stake.SyncStakeDelegations
import com.gemwallet.android.data.service.store.database.StakeDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.gemwallet.android.domains.asset.SYSTEM_VALIDATOR_ID
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext
import uniffi.gemstone.recommendedValidator
import uniffi.gemstone.recommendedValidatorIds
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.serializer.decodeJson
import uniffi.gemstone.GemStakeService
import java.math.BigInteger

class StakeRepository(
    private val stakeService: GemStakeService,
    private val stakeDao: StakeDao,
) : SyncStakeDelegations {
    override suspend fun sync(walletId: WalletId, assetId: AssetId, address: String) = withContext(Dispatchers.IO) {
        stakeService.sync(walletId.id, assetId.chain.string, address)
    }

    fun getRecommendValidators(assetId: AssetId): List<String> {
        return recommendedValidatorIds(assetId.chain.string)
    }

    fun getRecommended(assetId: AssetId): Flow<DelegationValidator?> {
        return getValidators(assetId).map { validators ->
            recommendedValidator(assetId.chain.string, validators.map { it.toJson() })?.decodeJson<DelegationValidator>()
        }
    }

    fun getValidators(assetId: AssetId): Flow<List<DelegationValidator>> {
        return stakeDao.getValidators(assetId, StakeProviderType.Stake)
            .map { items -> selectableValidators(items.toDTO()) }
    }

    fun getDelegations(walletId: WalletId, assetId: AssetId): Flow<List<Delegation>> {
        return stakeDao.getDelegations(walletId, assetId)
            .map { rows ->
                rows
                    .sortedByDescending { it.base.balance.toBigIntegerOrNull() ?: BigInteger.ZERO }
                    .mapNotNull { it.toModel() }
            }
    }

    fun getDelegation(walletId: WalletId, validatorId: String, delegationId: String): Flow<Delegation?> {
        return stakeDao.getDelegation(walletId, validatorId, delegationId).map { it?.toModel() }
    }

    suspend fun getRewards(walletId: WalletId, assetId: AssetId): List<Delegation> {
        return getDelegations(walletId, assetId).first()
            .filter { BigInteger(it.base.rewards) > BigInteger.ZERO }
    }

    suspend fun getStakeValidator(assetId: AssetId, validatorId: String): DelegationValidator? = withContext(Dispatchers.IO) {
        stakeDao.getValidator(assetId, validatorId)?.toDTO()
    }

}

internal fun selectableValidators(validators: List<DelegationValidator>): List<DelegationValidator> {
    return validators
        .filter { it.isActive && it.name.isNotEmpty() && it.id != SYSTEM_VALIDATOR_ID }
        .sortedByDescending { it.apr }
}
