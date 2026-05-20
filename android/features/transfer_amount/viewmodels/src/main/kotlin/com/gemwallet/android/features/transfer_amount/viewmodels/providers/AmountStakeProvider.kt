package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.coordinators.GetAssetInfo
import com.gemwallet.android.application.stake.coordinators.GetDelegation
import com.gemwallet.android.application.stake.coordinators.GetDelegations
import com.gemwallet.android.application.stake.coordinators.GetRecommendedValidator
import com.gemwallet.android.application.stake.coordinators.GetStakeValidator
import com.gemwallet.android.data.repositories.transactions.TransactionBalanceService
import com.gemwallet.android.domains.stake.hasRewards
import com.gemwallet.android.ext.byChain
import com.gemwallet.android.ext.changeAmountOnUnstake
import com.gemwallet.android.ext.freezed
import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.gemwallet.android.features.transfer_amount.models.ValidatorsSource
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.StakeChain
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
import uniffi.gemstone.Config
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class AmountStakeProvider(
    val params: AmountParams.Stake,
    getAssetInfo: GetAssetInfo,
    private val getDelegation: GetDelegation,
    private val getDelegations: GetDelegations,
    private val getRecommendedValidator: GetRecommendedValidator,
    private val getStakeValidator: GetStakeValidator,
    private val transactionBalanceService: TransactionBalanceService,
    scope: CoroutineScope,
) : AmountDataProvider {

    override val title: AmountTitle = AmountTitle.Stake(params)
    override val canSwitchInputType: Boolean = false

    private val stakeConfig by lazy { Config().getStakeConfig(params.assetId.chain.string) }

    override val canChangeValue: Boolean
        get() = when (params) {
            is AmountParams.Stake.Delegate,
            is AmountParams.Stake.Redelegate,
            is AmountParams.Stake.Freeze,
            is AmountParams.Stake.Unfreeze -> true
            is AmountParams.Stake.Undelegate -> params.assetId.chain.changeAmountOnUnstake
            is AmountParams.Stake.Withdraw,
            is AmountParams.Stake.Rewards -> false
        }

    override val showsAssetBalance: Boolean
        get() = when (params) {
            is AmountParams.Stake.Rewards -> true
            else -> canChangeValue
        }

    override val minimumValue: StateFlow<BigInteger> = MutableStateFlow(
        when (params) {
            is AmountParams.Stake.Delegate,
            is AmountParams.Stake.Freeze ->
                BigInteger.valueOf(stakeConfig.minAmount.toLong())
            is AmountParams.Stake.Redelegate -> when (StakeChain.byChain(params.assetId.chain)) {
                StakeChain.SmartChain -> BigInteger.valueOf(stakeConfig.minAmount.toLong())
                else -> BigInteger.ZERO
            }
            is AmountParams.Stake.Undelegate,
            is AmountParams.Stake.Withdraw,
            is AmountParams.Stake.Rewards,
            is AmountParams.Stake.Unfreeze -> BigInteger.ZERO
        }
    )

    override val reserveForFee: BigInteger
        get() = when (params) {
            is AmountParams.Stake.Delegate -> when (StakeChain.byChain(params.assetId.chain)?.freezed()) {
                true -> BigInteger.ZERO
                else -> BigInteger.valueOf(stakeConfig.reservedForFees.toLong())
            }
            is AmountParams.Stake.Freeze ->
                BigInteger.valueOf(stakeConfig.reservedForFees.toLong())
            is AmountParams.Stake.Undelegate,
            is AmountParams.Stake.Redelegate,
            is AmountParams.Stake.Withdraw,
            is AmountParams.Stake.Rewards,
            is AmountParams.Stake.Unfreeze -> BigInteger.ZERO
        }

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

    private val delegation: StateFlow<Delegation?> = run {
        val source = when (params) {
            is AmountParams.Stake.Undelegate ->
                getDelegation(validatorId = params.validatorId, delegationId = params.delegationId)
            is AmountParams.Stake.Redelegate ->
                getDelegation(validatorId = params.validatorId, delegationId = params.delegationId)
            is AmountParams.Stake.Withdraw ->
                getDelegation(validatorId = params.validatorId, delegationId = params.delegationId)
            is AmountParams.Stake.Rewards ->
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

    override val availableBalance: StateFlow<BigInteger> =
        combine(assetInfo.filterNotNull(), delegation, selectedResource) { current, currentDelegation, currentResource ->
            transactionBalanceService.getBalance(current, params, delegation = currentDelegation, resource = currentResource)
        }
            .flowOn(Dispatchers.IO)
            .stateIn(scope, SharingStarted.Eagerly, BigInteger.ZERO)

    override fun shouldReserveFee(isMaxAmount: Boolean): Boolean {
        if (!isMaxAmount || reserveForFee.signum() == 0) return false
        return when (params) {
            is AmountParams.Stake.Delegate, is AmountParams.Stake.Freeze -> {
                val maxAfterFee = (availableBalance.value - reserveForFee).max(BigInteger.ZERO)
                maxAfterFee > minimumValue.value
            }
            else -> false
        }
    }

    override suspend fun buildConfirmParams(amount: Crypto, isMax: Boolean): ConfirmParams {
        val current = assetInfo.value ?: error("assetInfo not loaded")
        val owner = current.owner ?: error("owner missing")
        val builder = ConfirmParams.Builder(current.asset, owner, amount.atomicValue, isMax)
        return when (params) {
            is AmountParams.Stake.Delegate -> builder.delegate(currentValidator)
            is AmountParams.Stake.Redelegate -> builder.redelegate(currentValidator, currentDelegation)
            is AmountParams.Stake.Undelegate -> builder.undelegate(currentDelegation)
            is AmountParams.Stake.Withdraw -> builder.withdraw(currentDelegation)
            is AmountParams.Stake.Rewards -> builder.rewards(listOf(currentValidator))
            is AmountParams.Stake.Freeze -> builder.freeze(selectedResource.value)
            is AmountParams.Stake.Unfreeze -> builder.unfreeze(selectedResource.value)
        }
    }

    private val currentValidator: DelegationValidator
        get() = validatorState.value ?: throw AmountError.NoValidatorSelected

    private val currentDelegation: Delegation
        get() = delegation.value ?: throw AmountError.NoDelegationSelected

}
