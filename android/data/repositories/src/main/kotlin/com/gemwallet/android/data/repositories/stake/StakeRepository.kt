package com.gemwallet.android.data.repositories.stake

import com.gemwallet.android.blockchain.services.StakeService
import com.gemwallet.android.cases.stake.SyncStakeDelegations
import com.gemwallet.android.data.service.store.database.StakeDao
import com.gemwallet.android.domains.asset.SYSTEM_VALIDATOR_ID
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.services.gemapi.GemApiStaticClient
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationState
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext
import uniffi.gemstone.Config
import java.math.BigInteger

class StakeRepository(
    private val gemApiStaticClient: GemApiStaticClient,
    private val stakeService: StakeService,
    private val stakeDao: StakeDao,
) : SyncStakeDelegations {
    private val recommendedValidators = Config().getValidators()

    override suspend fun sync(walletId: WalletId, assetId: AssetId, address: String, apr: Double) = withContext(Dispatchers.IO) {
        syncValidators(assetId, address, apr)
        syncDelegations(walletId, assetId, address)
    }

    fun getRecommendValidators(assetId: AssetId): List<String> {
        return recommendedValidators[assetId.chain.string].orEmpty()
    }

    fun getRecommended(assetId: AssetId): Flow<DelegationValidator?> {
        val recommendedIds = getRecommendValidators(assetId)
        return getValidators(assetId).map { pickRecommendedValidator(it, recommendedIds) }
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

    fun getDelegation(validatorId: String, delegationId: String): Flow<Delegation?> {
        return stakeDao.getDelegation(validatorId, delegationId).map { it?.toModel() }
    }

    suspend fun getRewards(walletId: WalletId, assetId: AssetId): List<Delegation> {
        return getDelegations(walletId, assetId).first()
            .filter { BigInteger(it.base.rewards) > BigInteger.ZERO }
    }

    suspend fun getStakeValidator(assetId: AssetId, validatorId: String): DelegationValidator? = withContext(Dispatchers.IO) {
        stakeDao.getValidator(assetId, validatorId)?.toDTO()
    }

    private suspend fun syncValidators(assetId: AssetId, address: String, apr: Double) {
        val chain = assetId.chain
        val names = runCatching { gemApiStaticClient.getValidators(chain.string) }
            .getOrDefault(emptyList())
            .associateBy { it.id }
        val validators = stakeService.getValidators(chain, apr)
        val validatorIds = validators.map { it.id }.toSet()
        val delegationValidators = stakeService.getDelegationValidators(chain, address)
            .filterNot { validatorIds.contains(it.id) }
        val updateValidators = (validators + delegationValidators).map { validator ->
            if (validator.name.isEmpty()) validator.copy(name = names[validator.id]?.name.orEmpty()) else validator
        }
        stakeDao.upsertValidators(updateValidators.toRecord())
    }

    private suspend fun syncDelegations(walletId: WalletId, assetId: AssetId, address: String) {
        val delegations = runCatching { stakeService.getStakeDelegations(assetId.chain, address) }
            .getOrNull() ?: return
        val incoming = delegations.toRecord(walletId)
        val incomingIds = incoming.map { it.id }.toSet()
        val deleteIds = stakeDao.getDelegationIds(walletId, assetId)
            .filterNot { incomingIds.contains(it) }
        val validators = stakeDao.getValidators(assetId, StakeProviderType.Stake)
            .first()
            .toDTO()
            .associateBy { it.id }
        val upsertable = incoming.mapNotNull { delegation ->
            val validator = validators[delegation.validatorId] ?: return@mapNotNull null
            if (delegation.state == DelegationState.Active && !validator.isActive) {
                delegation.copy(state = DelegationState.Inactive)
            } else {
                delegation
            }
        }
        stakeDao.updateAndDeleteDelegations(walletId, upsertable, deleteIds)
    }
}

internal fun pickRecommendedValidator(
    validators: List<DelegationValidator>,
    recommendedIds: Collection<String>,
): DelegationValidator? {
    return validators
        .filter { recommendedIds.contains(it.id) }
        .randomOrNull()
        ?: validators.firstOrNull()
}

internal fun selectableValidators(validators: List<DelegationValidator>): List<DelegationValidator> {
    return validators
        .filter { it.isActive && it.name.isNotEmpty() && it.id != SYSTEM_VALIDATOR_ID }
        .sortedByDescending { it.apr }
}
