package com.gemwallet.android.data.coordinators.stake

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.application.stake.cases.GetDelegation
import com.gemwallet.android.application.stake.cases.GetDelegations
import com.gemwallet.android.application.stake.cases.GetRecommendedValidator
import com.gemwallet.android.application.stake.cases.GetStakeValidator
import com.gemwallet.android.application.stake.cases.GetValidators
import com.gemwallet.android.application.stake.cases.SyncStakeDelegations
import com.gemwallet.android.data.services.gemstone.stores.GemstoneStakeStore
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemStakeService
import uniffi.gemstone.GemStakeServiceInterface
import java.math.BigInteger

class SyncStakeDelegationsImpl(
    private val stakeService: GemStakeService,
) : SyncStakeDelegations {

    override suspend fun sync(chain: Chain) = withContext(Dispatchers.IO) {
        stakeService.sync(chain.string)
    }
}

class GetValidatorsImpl(
    private val stakeStore: GemstoneStakeStore,
    private val stakeService: GemStakeServiceInterface,
) : GetValidators {

    override fun invoke(assetId: AssetId): Flow<List<DelegationValidator>> =
        stakeStore.observeValidators(assetId, StakeProviderType.Stake)
            .map { validators -> stakeService.selectableValidators(validators.map { it.toGem() }).map { it.toPrimitives() } }
}

class GetRecommendedValidatorImpl(
    private val getValidators: GetValidators,
    private val stakeService: GemStakeServiceInterface,
) : GetRecommendedValidator {

    override fun invoke(assetId: AssetId): Flow<DelegationValidator?> =
        getValidators(assetId).map { validators ->
            stakeService.recommendedValidator(assetId.chain.string, validators.map { it.toGem() })?.toPrimitives()
        }
}

class GetDelegationsImpl(
    private val stakeStore: GemstoneStakeStore,
) : GetDelegations {

    override fun invoke(walletId: WalletId, assetId: AssetId): Flow<List<Delegation>> =
        stakeStore.observeDelegations(walletId, assetId)
            .map { delegations -> delegations.sortedByDescending { it.base.balance } }
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
