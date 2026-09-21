package com.gemwallet.android.data.coordinators.stake

import com.gemwallet.android.application.stake.cases.GetDelegation
import com.gemwallet.android.application.stake.cases.GetDelegations
import com.gemwallet.android.application.stake.cases.GetStakeValidator
import com.gemwallet.android.application.stake.cases.GetValidators
import com.gemwallet.android.data.services.gemstone.stores.GemstoneStakeStore
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemStakeServiceInterface
import java.math.BigInteger

class GetValidatorsImpl(private val stakeStore: GemstoneStakeStore, private val stakeService: GemStakeServiceInterface, private val ioDispatcher: CoroutineDispatcher) : GetValidators {

    override fun invoke(assetId: AssetId, providerType: StakeProviderType): Flow<List<DelegationValidator>> = stakeStore.observeValidators(assetId, providerType)
        .map { validators -> stakeService.selectableValidators(validators.map { it.toGem() }).map { it.toPrimitives() } }
        .flowOn(ioDispatcher)
}

class GetDelegationsImpl(private val stakeStore: GemstoneStakeStore, private val stakeService: GemStakeServiceInterface, private val ioDispatcher: CoroutineDispatcher) : GetDelegations {

    override fun invoke(walletId: WalletId, assetId: AssetId, providerType: StakeProviderType): Flow<List<Delegation>> = stakeStore.observeDelegations(walletId, assetId, providerType)
        .map { delegations -> stakeService.sortedDelegations(delegations.map { it.toGem() }).map { it.toPrimitives() } }
        .flowOn(ioDispatcher)
}

class GetDelegationImpl(private val stakeStore: GemstoneStakeStore) : GetDelegation {

    override fun invoke(walletId: WalletId, validatorId: String, delegationId: String): Flow<Delegation?> = stakeStore.observeDelegation(walletId, validatorId, delegationId)
}

class GetStakeValidatorImpl(private val stakeStore: GemstoneStakeStore) : GetStakeValidator {

    override suspend fun invoke(assetId: AssetId, validatorId: String): DelegationValidator? = withContext(Dispatchers.IO) {
        stakeStore.getValidator(assetId, validatorId)
    }
}
