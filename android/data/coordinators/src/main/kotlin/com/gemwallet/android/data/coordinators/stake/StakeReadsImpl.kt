package com.gemwallet.android.data.coordinators.stake

import com.gemwallet.android.application.stake.cases.GetDelegation
import com.gemwallet.android.application.stake.cases.GetDelegations
import com.gemwallet.android.application.stake.cases.GetStakeValidator
import com.gemwallet.android.application.stake.cases.GetValidators
import com.gemwallet.android.data.services.gemstone.stores.GemstoneStakeStore
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.withContext

class GetValidatorsImpl(private val stakeStore: GemstoneStakeStore) : GetValidators {

    override fun invoke(assetId: AssetId, providerType: StakeProviderType): Flow<List<DelegationValidator>> = stakeStore.observeValidators(assetId, providerType)
}

class GetDelegationsImpl(private val stakeStore: GemstoneStakeStore) : GetDelegations {

    override fun invoke(walletId: WalletId, assetId: AssetId, providerType: StakeProviderType): Flow<List<Delegation>> = stakeStore.observeDelegations(walletId, assetId, providerType)
}

class GetDelegationImpl(private val stakeStore: GemstoneStakeStore) : GetDelegation {

    override fun invoke(walletId: WalletId, validatorId: String, delegationId: String): Flow<Delegation?> = stakeStore.observeDelegation(walletId, validatorId, delegationId)
}

class GetStakeValidatorImpl(private val stakeStore: GemstoneStakeStore) : GetStakeValidator {

    override suspend fun invoke(assetId: AssetId, validatorId: String): DelegationValidator? = withContext(Dispatchers.IO) {
        stakeStore.getValidator(assetId, validatorId)
    }
}
