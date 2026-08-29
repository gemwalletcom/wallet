package com.gemwallet.android.data.coordinators.stake

import com.gemwallet.android.application.stake.cases.GetDelegation
import com.gemwallet.android.application.stake.cases.GetDelegations
import com.gemwallet.android.application.stake.cases.GetRecommendedValidator
import com.gemwallet.android.application.stake.cases.GetRecommendedValidatorIds
import com.gemwallet.android.application.stake.cases.GetStakeValidator
import com.gemwallet.android.application.stake.cases.GetValidators
import com.gemwallet.android.cases.stake.SyncStakeDelegations
import com.gemwallet.android.data.service.store.database.StakeDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemStakeRulesService
import uniffi.gemstone.GemStakeService
import java.math.BigInteger

class SyncStakeDelegationsImpl(
    private val stakeService: GemStakeService,
) : SyncStakeDelegations {

    override suspend fun sync(walletId: WalletId, assetId: AssetId, address: String) = withContext(Dispatchers.IO) {
        stakeService.sync(walletId.id, assetId.chain.string, address)
    }
}

class GetValidatorsImpl(
    private val stakeDao: StakeDao,
    private val stakeRules: GemStakeRulesService,
) : GetValidators {

    override fun invoke(assetId: AssetId): Flow<List<DelegationValidator>> =
        stakeDao.getValidators(assetId, StakeProviderType.Stake)
            .map { items -> stakeRules.selectableValidators(items.toDTO().map { it.toJson() }).map { it.decodeJson<DelegationValidator>() } }
}

class GetRecommendedValidatorIdsImpl(
    private val stakeRules: GemStakeRulesService,
) : GetRecommendedValidatorIds {

    override fun invoke(assetId: AssetId): List<String> = stakeRules.recommendedValidatorIds(assetId.chain.string)
}

class GetRecommendedValidatorImpl(
    private val getValidators: GetValidators,
    private val stakeRules: GemStakeRulesService,
) : GetRecommendedValidator {

    override fun invoke(assetId: AssetId): Flow<DelegationValidator?> =
        getValidators(assetId).map { validators ->
            stakeRules.recommendedValidator(assetId.chain.string, validators.map { it.toJson() })?.decodeJson<DelegationValidator>()
        }
}

class GetDelegationsImpl(
    private val stakeDao: StakeDao,
) : GetDelegations {

    override fun invoke(walletId: WalletId, assetId: AssetId): Flow<List<Delegation>> =
        stakeDao.getDelegations(walletId, assetId)
            .map { rows ->
                rows
                    .sortedByDescending { it.base.balance.toBigIntegerOrNull() ?: BigInteger.ZERO }
                    .mapNotNull { it.toModel() }
            }
}

class GetDelegationImpl(
    private val stakeDao: StakeDao,
) : GetDelegation {

    override fun invoke(walletId: WalletId, validatorId: String, delegationId: String): Flow<Delegation?> =
        stakeDao.getDelegation(walletId, validatorId, delegationId).map { it?.toModel() }
}

class GetStakeValidatorImpl(
    private val stakeDao: StakeDao,
) : GetStakeValidator {

    override suspend fun invoke(assetId: AssetId, validatorId: String): DelegationValidator? = withContext(Dispatchers.IO) {
        stakeDao.getValidator(assetId, validatorId)?.toDTO()
    }
}
