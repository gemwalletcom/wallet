package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.stake.cases.GetDelegation
import com.gemwallet.android.application.stake.cases.GetDelegations
import com.gemwallet.android.application.stake.cases.GetRecommendedValidator
import com.gemwallet.android.application.stake.cases.GetStakeValidator
import com.gemwallet.android.domains.stake.hasRewards
import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.gemwallet.android.features.transfer_amount.models.ValidatorsSource
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.Resource
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemAmountStakeType
import uniffi.gemstone.GemAmountType
import uniffi.gemstone.GemTransferData
import com.wallet.core.primitives.RedelegateData
import com.wallet.core.primitives.StakeType
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.serializer.toJson

@OptIn(ExperimentalCoroutinesApi::class)
class AmountStakeProvider(
    val params: AmountParams.Stake,
    getAssetInfo: GetAssetInfo,
    private val getDelegation: GetDelegation,
    private val getDelegations: GetDelegations,
    private val getRecommendedValidator: GetRecommendedValidator,
    private val getStakeValidator: GetStakeValidator,
    private val service: GemAmountServiceInterface,
    scope: CoroutineScope,
) : AmountDataProvider(scope) {

    override val title: AmountTitle = AmountTitle.Stake(params)
    override val canSwitchInputType: Boolean = false

    override val assetInfo: StateFlow<AssetInfo?> =
        getAssetInfo(params.assetId)
            .flowOn(Dispatchers.IO)
            .stateIn(scope, SharingStarted.Eagerly, null)

    private val selectedValidatorId = MutableStateFlow<String?>(
        when (params) {
            is AmountParams.Stake.Delegate -> params.validatorId
            is AmountParams.Stake.Redelegate -> params.validatorId
            else -> null
        }
    )

    private val selectedResource = MutableStateFlow(
        when (params) {
            is AmountParams.Stake.Freeze -> params.resource
            is AmountParams.Stake.Unfreeze -> params.resource
            else -> Resource.Bandwidth
        }
    )
    val resource: StateFlow<Resource> = selectedResource.asStateFlow()

    fun setResource(value: Resource) {
        selectedResource.update { value }
    }

    private val rewardsDelegations: StateFlow<List<Delegation>> = when (params) {
        is AmountParams.Stake.Rewards -> assetInfo.filterNotNull().flatMapLatest { current ->
            val walletId = current.walletId ?: return@flatMapLatest flowOf(emptyList())
            getDelegations(walletId, current.asset.id).map { list -> list.filter { it.hasRewards() } }
        }.flowOn(Dispatchers.IO).stateIn(scope, SharingStarted.Eagerly, emptyList())
        else -> MutableStateFlow(emptyList())
    }

    private data class DelegationIdentity(val validatorId: String, val delegationId: String)

    private val delegationIdentity: DelegationIdentity? = when (params) {
        is AmountParams.Stake.Undelegate -> DelegationIdentity(params.validatorId, params.delegationId)
        is AmountParams.Stake.Redelegate -> DelegationIdentity(params.validatorId, params.delegationId)
        is AmountParams.Stake.Withdraw -> DelegationIdentity(params.validatorId, params.delegationId)
        else -> null
    }

    private val delegation: StateFlow<Delegation?> = run {
        val source = when {
            delegationIdentity != null -> assetInfo.filterNotNull().flatMapLatest { current ->
                val walletId = current.walletId ?: return@flatMapLatest flowOf(null)
                getDelegation(
                    walletId = walletId,
                    validatorId = delegationIdentity.validatorId,
                    delegationId = delegationIdentity.delegationId,
                )
            }
            params is AmountParams.Stake.Rewards ->
                combine(rewardsDelegations, selectedValidatorId) { withRewards, pickedId ->
                    withRewards.firstOrNull { it.validator.id == pickedId } ?: withRewards.firstOrNull()
                }
            else -> flowOf(null)
        }
        source.flowOn(Dispatchers.IO).stateIn(scope, SharingStarted.Eagerly, null)
    }

    private val recommendedValidator: StateFlow<DelegationValidator?> = when (params) {
        is AmountParams.Stake.Delegate,
        is AmountParams.Stake.Redelegate -> getRecommendedValidator(params.assetId)
            .flowOn(Dispatchers.IO)
            .stateIn(scope, SharingStarted.Eagerly, null)
        else -> MutableStateFlow(null)
    }

    val validatorState: StateFlow<DelegationValidator?> =
        combine(assetInfo, delegation, selectedValidatorId, recommendedValidator) { current, currentDelegation, pickedId, recommended ->
            val byId = if (current != null && pickedId != null) {
                getStakeValidator(current.asset.id, pickedId)
            } else {
                null
            }
            byId ?: currentDelegation?.validator ?: recommended
        }.flowOn(Dispatchers.IO).stateIn(scope, SharingStarted.Eagerly, null)

    val validatorSource: StateFlow<ValidatorsSource?> = assetInfo.mapLatest { current ->
        when (params) {
            is AmountParams.Stake.Rewards ->
                current?.walletId?.let { ValidatorsSource.Rewards(walletId = it, assetId = params.assetId) }
            is AmountParams.Stake.Freeze, is AmountParams.Stake.Unfreeze -> null
            is AmountParams.Stake.Delegate,
            is AmountParams.Stake.Redelegate,
            is AmountParams.Stake.Undelegate,
            is AmountParams.Stake.Withdraw -> ValidatorsSource.ChainValidators(assetId = params.assetId)
        }
    }.stateIn(scope, SharingStarted.Eagerly, null)

    val canSelectValidator: StateFlow<Boolean> = when (params) {
        is AmountParams.Stake.Delegate,
        is AmountParams.Stake.Redelegate -> MutableStateFlow(true).asStateFlow()
        is AmountParams.Stake.Rewards -> rewardsDelegations
            .map { it.size > 1 }
            .stateIn(scope, SharingStarted.Eagerly, false)
        is AmountParams.Stake.Undelegate,
        is AmountParams.Stake.Withdraw,
        is AmountParams.Stake.Freeze,
        is AmountParams.Stake.Unfreeze -> MutableStateFlow(false).asStateFlow()
    }

    fun selectValidator(id: String?) {
        selectedValidatorId.update { id }
    }

    override val amountType: StateFlow<GemAmountType?> =
        combine(delegation, selectedResource) { currentDelegation, currentResource ->
            val stakeType = when (params) {
                is AmountParams.Stake.Delegate -> GemAmountStakeType.Stake
                is AmountParams.Stake.Undelegate -> currentDelegation?.let { GemAmountStakeType.Unstake(it.toJson()) }
                is AmountParams.Stake.Redelegate -> currentDelegation?.let { GemAmountStakeType.Redelegate(it.toJson()) }
                is AmountParams.Stake.Withdraw -> currentDelegation?.let { GemAmountStakeType.Withdraw(it.toJson()) }
                is AmountParams.Stake.Rewards -> GemAmountStakeType.Rewards(listOfNotNull(currentDelegation).map { it.toJson() })
                is AmountParams.Stake.Freeze -> GemAmountStakeType.Freeze(currentResource.toJson())
                is AmountParams.Stake.Unfreeze -> GemAmountStakeType.Unfreeze(currentResource.toJson())
            }
            stakeType?.let { GemAmountType.Stake(it) }
        }.stateIn(scope, SharingStarted.Eagerly, null)

    override suspend fun buildTransfer(amount: Crypto, isMax: Boolean): GemTransferData {
        val current = assetInfo.value ?: error("assetInfo not loaded")
        val stakeType: StakeType = when (params) {
            is AmountParams.Stake.Delegate -> StakeType.Stake(currentValidator)
            is AmountParams.Stake.Redelegate -> StakeType.Redelegate(RedelegateData(currentDelegation, currentValidator))
            is AmountParams.Stake.Undelegate -> StakeType.Unstake(currentDelegation)
            is AmountParams.Stake.Withdraw -> StakeType.Withdraw(currentDelegation)
            is AmountParams.Stake.Rewards -> StakeType.Rewards(listOf(currentValidator))
            is AmountParams.Stake.Freeze -> StakeType.Freeze(selectedResource.value)
            is AmountParams.Stake.Unfreeze -> StakeType.Unfreeze(selectedResource.value)
        }
        return service.stakeTransferData(current.asset.toGem(), stakeType.toJson(), amount.atomicValue.toString(), isMax)
    }

    private val currentValidator: DelegationValidator
        get() = validatorState.value ?: throw AmountError.NoValidatorSelected

    private val currentDelegation: Delegation
        get() = delegation.value ?: throw AmountError.NoDelegationSelected

}
