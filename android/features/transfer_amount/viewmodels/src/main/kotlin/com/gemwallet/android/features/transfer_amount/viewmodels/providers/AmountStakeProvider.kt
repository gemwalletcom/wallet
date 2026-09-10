package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.stake.cases.GetDelegation
import com.gemwallet.android.application.stake.cases.GetDelegations
import com.gemwallet.android.application.stake.cases.GetStakeValidator
import com.gemwallet.android.application.stake.cases.GetValidators
import com.gemwallet.android.domains.stake.hasRewards
import com.gemwallet.android.features.transfer_amount.models.AmountError
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
import uniffi.gemstone.GemAmountType
import uniffi.gemstone.GemStakeAmountInput
import uniffi.gemstone.GemStakeValidatorSelection
import uniffi.gemstone.GemTransferData
import com.wallet.core.primitives.RedelegateData
import com.wallet.core.primitives.StakeType
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives

@OptIn(ExperimentalCoroutinesApi::class)
class AmountStakeProvider(
    val params: AmountParams.Stake,
    getAssetInfo: GetAssetInfo,
    private val getDelegation: GetDelegation,
    private val getDelegations: GetDelegations,
    private val getStakeValidator: GetStakeValidator,
    getValidators: GetValidators,
    private val service: GemAmountServiceInterface,
    scope: CoroutineScope,
) : AmountDataProvider(scope) {

    override val title: AmountTitle = AmountTitle.Stake(params)

    override val assetInfo: StateFlow<AssetInfo?> =
        getAssetInfo(params.assetId)
            .flowOn(Dispatchers.IO)
            .stateIn(scope, SharingStarted.Eagerly, null)

    private val selectedValidatorId = MutableStateFlow<String?>(
        when (params) {
            is AmountParams.Stake.Delegate -> params.validatorId
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

    val validatorSelection: StateFlow<GemStakeValidatorSelection?> =
        combine(getValidators(params.assetId), delegation, rewardsDelegations) { validators, currentDelegation, rewards ->
            validatorInput(validators, currentDelegation, rewards)?.let { service.stakeValidatorSelection(params.assetId.chain.string, it) }
        }.flowOn(Dispatchers.IO).stateIn(scope, SharingStarted.Eagerly, null)

    val validatorState: StateFlow<DelegationValidator?> =
        combine(assetInfo, selectedValidatorId, validatorSelection) { current, pickedId, selection ->
            val byId = if (current != null && pickedId != null) {
                getStakeValidator(current.asset.id, pickedId)
            } else {
                null
            }
            byId ?: selection?.validator?.toPrimitives()
        }.flowOn(Dispatchers.IO).stateIn(scope, SharingStarted.Eagerly, null)

    private fun validatorInput(
        validators: List<DelegationValidator>,
        delegation: Delegation?,
        rewards: List<Delegation>,
    ): GemStakeAmountInput? = when (params) {
        is AmountParams.Stake.Delegate -> GemStakeAmountInput.Stake(validators.map { it.toGem() }, delegation = null)
        is AmountParams.Stake.Redelegate -> delegation?.let { GemStakeAmountInput.Redelegate(validators.map { validator -> validator.toGem() }, it.toGem()) }
        is AmountParams.Stake.Undelegate -> delegation?.let { GemStakeAmountInput.Unstake(it.toGem()) }
        is AmountParams.Stake.Withdraw -> delegation?.let { GemStakeAmountInput.Withdraw(it.toGem()) }
        is AmountParams.Stake.Rewards -> GemStakeAmountInput.Rewards(rewards.map { it.toGem() })
        is AmountParams.Stake.Freeze -> GemStakeAmountInput.Freeze(params.resource.toGem())
        is AmountParams.Stake.Unfreeze -> GemStakeAmountInput.Unfreeze(params.resource.toGem())
    }

    val canSelectValidator: StateFlow<Boolean> = validatorSelection
        .map { it?.canSelect == true }
        .stateIn(scope, SharingStarted.Eagerly, false)

    fun selectValidator(id: String?) {
        selectedValidatorId.update { id }
    }

    override val amountType: StateFlow<GemAmountType?> =
        combine(delegation, validatorState, selectedResource, rewardsDelegations) { currentDelegation, currentValidator, currentResource, rewards ->
            stakeType(currentDelegation, currentValidator, currentResource)?.let { service.stakeAmountType(it.toGem(), rewards.map { delegation -> delegation.toGem() }) }
        }.stateIn(scope, SharingStarted.Eagerly, null)

    override suspend fun buildTransfer(amount: Crypto, isMax: Boolean): GemTransferData {
        val current = assetInfo.value ?: error("assetInfo not loaded")
        val stakeType = stakeType(delegation.value, validatorState.value, selectedResource.value) ?: throw missingSelection()
        return service.stakeTransferData(current.asset.toGem(), stakeType.toGem(), amount.atomicValue, isMax)
    }

    private fun stakeType(delegation: Delegation?, validator: DelegationValidator?, resource: Resource): StakeType? = when (params) {
        is AmountParams.Stake.Delegate -> validator?.let { StakeType.Stake(it) }
        is AmountParams.Stake.Redelegate -> if (delegation != null && validator != null) StakeType.Redelegate(RedelegateData(delegation, validator)) else null
        is AmountParams.Stake.Undelegate -> delegation?.let { StakeType.Unstake(it) }
        is AmountParams.Stake.Withdraw -> delegation?.let { StakeType.Withdraw(it) }
        is AmountParams.Stake.Rewards -> validator?.let { StakeType.Rewards(listOf(it)) }
        is AmountParams.Stake.Freeze -> StakeType.Freeze(resource)
        is AmountParams.Stake.Unfreeze -> StakeType.Unfreeze(resource)
    }

    private fun missingSelection(): AmountError = when (params) {
        is AmountParams.Stake.Delegate,
        is AmountParams.Stake.Rewards -> AmountError.NoValidatorSelected
        is AmountParams.Stake.Redelegate -> if (delegation.value == null) AmountError.NoDelegationSelected else AmountError.NoValidatorSelected
        is AmountParams.Stake.Undelegate,
        is AmountParams.Stake.Withdraw,
        is AmountParams.Stake.Freeze,
        is AmountParams.Stake.Unfreeze -> AmountError.NoDelegationSelected
    }

}
