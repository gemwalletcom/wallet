package com.gemwallet.android.data.coordinators.stake

import com.gemwallet.android.application.stake.cases.GetDelegation
import com.gemwallet.android.application.stake.cases.GetDelegations
import com.gemwallet.android.application.stake.cases.GetRecommendedValidator
import com.gemwallet.android.application.stake.cases.GetRecommendedValidatorIds
import com.gemwallet.android.application.stake.cases.GetStakeValidator
import com.gemwallet.android.application.stake.cases.GetValidators
import com.gemwallet.android.application.stake.cases.SyncStakeDelegations
import com.gemwallet.android.data.services.gemstone.stores.GemstoneStakeStore
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
import uniffi.gemstone.GemStakeConfigService
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
    private val stakeStore: GemstoneStakeStore,
    private val stakeConfig: GemStakeConfigService,
) : GetValidators {

    override fun invoke(assetId: AssetId): Flow<List<DelegationValidator>> =
        stakeStore.observeValidators(assetId, StakeProviderType.Stake)
            .map { validators -> stakeConfig.selectableValidators(validators.map { it.toJson() }).map { it.decodeJson<DelegationValidator>() } }
}

class GetRecommendedValidatorIdsImpl(
    private val stakeConfig: GemStakeConfigService,
) : GetRecommendedValidatorIds {

    override fun invoke(assetId: AssetId): List<String> = stakeConfig.recommendedValidatorIds(assetId.chain.string)
}

class GetRecommendedValidatorImpl(
    private val getValidators: GetValidators,
    private val stakeConfig: GemStakeConfigService,
) : GetRecommendedValidator {

    override fun invoke(assetId: AssetId): Flow<DelegationValidator?> =
        getValidators(assetId).map { validators ->
            stakeConfig.recommendedValidator(assetId.chain.string, validators.map { it.toJson() })?.decodeJson<DelegationValidator>()
        }
}

class GetDelegationsImpl(
    private val stakeStore: GemstoneStakeStore,
) : GetDelegations {

    override fun invoke(walletId: WalletId, assetId: AssetId): Flow<List<Delegation>> =
        stakeStore.observeDelegations(walletId, assetId)
            .map { delegations -> delegations.sortedByDescending { it.base.balance.toBigIntegerOrNull() ?: BigInteger.ZERO } }
}

class GetDelegationImpl(
    private val stakeStore: GemstoneStakeStore,
) : GetDelegation {

    override fun invoke(walletId: WalletId, validatorId: String, delegationId: String): Flow<Delegation?> =
        stakeStore.observeDelegation(walletId, validatorId, delegationId)
}

class GetStakeValidatorImpl(
    private val stakeStore: GemstoneStakeStore,
) : GetStakeValidator {

    override suspend fun invoke(assetId: AssetId, validatorId: String): DelegationValidator? = withContext(Dispatchers.IO) {
        stakeStore.getValidator(assetId, validatorId)
    }
}
