package com.gemwallet.android.features.stake.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.gemwallet.android.domains.stake.hasRewards
import com.gemwallet.android.domains.stake.rewardsBalance
import com.gemwallet.android.domains.stake.sumRewardsBalance
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.stakeChain
import com.gemwallet.android.AppUrl
import com.gemwallet.android.ext.claimAllAvailable
import com.gemwallet.android.ext.canClaimRewards
import com.gemwallet.android.ext.freezed
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toGemStakeChain
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.format
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.features.stake.models.StakeAction
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationState
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.WalletType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import uniffi.gemstone.DocsUrl
import java.math.BigInteger
import javax.inject.Inject

private const val assetIdArg = "assetId"

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class StakeViewModel @Inject constructor(
    private val assetsRepository: AssetsRepository,
    private val stakeRepository: StakeRepository,
    sessionRepository: SessionRepository,
    stateHandle: SavedStateHandle,
): ViewModel() {
    private val initialAssetId = stateHandle.get<String>(assetIdArg)?.toAssetId()
        ?: error("Missing assetId")

    private val assetId = stateHandle.getStateFlow(assetIdArg, initialAssetId.toIdentifier())
        .map { it.toAssetId() ?: initialAssetId }
        .stateIn(viewModelScope, SharingStarted.Eagerly, initialAssetId)

    val assetInfo = assetId
        .flatMapLatest { assetsRepository.getAssetInfo(it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val stakeInfoUrl = assetInfo
        .mapLatest { it?.stakeChain?.let { chain -> AppUrl.docs(DocsUrl.Staking(chain.toGemStakeChain())) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val session = sessionRepository.session()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val walletType = session.mapLatest { it?.wallet?.type }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val account = session.combine(assetId) { session, assetId ->
        session?.wallet?.getAccount(assetId.chain)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val delegations = account.combine(assetId) { account, assetId -> Pair(account, assetId) }
        .flatMapLatest {
            val (account, assetId) = it
            val accountAddress = account?.address ?: return@flatMapLatest emptyFlow()
            stakeRepository.getDelegations(assetId, accountAddress)
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val isStakeEnabled = assetId
        .flatMapLatest { stakeRepository.getValidators(it.chain) }
        .mapLatest { validators -> validators.isNotEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val rewardsBalance = delegations
        .mapLatest { delegations -> delegations.sumRewardsBalance() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, BigInteger.ZERO)

    val actions = combine(
        session.mapLatest { it?.wallet?.type }.filterNotNull(),
        rewardsBalance,
        assetInfo.filterNotNull(),
    ) { walletType, rewardsBalance, assetInfo ->
        if (walletType == WalletType.View) {
            return@combine emptyList()
        }
        listOfNotNull(
            StakeAction.Stake,
            StakeAction.Freeze.takeIf { assetInfo.stakeChain?.freezed() == true },
            StakeAction.Unfreeze.takeIf { assetInfo.stakeChain?.freezed() == true },
            rewardsBalance
                .takeIf { assetInfo.chain.canClaimRewards && rewardsBalance > BigInteger.ZERO }
                ?.let {
                    StakeAction.Rewards(
                        data = assetInfo.asset.format(
                            crypto = Crypto(rewardsBalance),
                            decimalPlace = 2,
                            maxDecimals = assetInfo.asset.decimals,
                            dynamicPlace = true,
                        ),
                    )
                },
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val sync = MutableStateFlow<Boolean>(true)

    val isSync = combine(sync, assetInfo.filterNotNull(), account.filterNotNull()) { sync, assetInfo, account ->
        Triple(sync, assetInfo, account)
    }
        .flatMapLatest {
            val (isSync, assetInfo, account) = it
            flow {
                if (!isSync) {
                    emit(false)
                    return@flow
                }
                emit(true)
                stakeRepository.sync(assetInfo.asset.chain, account.address, assetInfo.stakeApr ?: 0.0)
                emit(false)
                sync.update { false }
            }
        }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, true)

    fun onRefresh() {
        sync.update { true }
    }

    fun onDelegation(
        delegation: Delegation,
        onOpenDetail: (String, String) -> Unit,
        onConfirm: ConfirmTransactionAction,
    ) {
        if (delegation.base.state != DelegationState.AwaitingWithdrawal) {
            onOpenDetail(delegation.validator.id, delegation.base.delegationId)
            return
        }
        val assetInfo = assetInfo.value ?: return
        val from = assetInfo.owner ?: return
        val balance = Crypto(delegation.base.balance.toBigIntegerOrNull() ?: BigInteger.ZERO)
        val params = ConfirmParams.Builder(assetInfo.asset, from, balance.atomicValue, false)
            .withdraw(delegation)
        onConfirm(params)
    }

    fun onRewards(onAmount: AmountTransactionAction, onConfirm: ConfirmTransactionAction) {
        val assetInfo = assetInfo.value ?: return
        val account = account.value ?: return
        val withRewards = delegations.value.filter { it.hasRewards() }
        val canClaimAllRewards = assetInfo.chain.claimAllAvailable || withRewards.size == 1
        if (canClaimAllRewards) {
            onConfirm(
                ConfirmParams.Stake.RewardsParams(
                    asset = assetInfo.asset,
                    from = account,
                    validators = withRewards.map { it.validator },
                    amount = withRewards.sumOf { it.rewardsBalance() },
                )
            )
        } else {
            onAmount(
                AmountParams.buildStake(
                    assetId = assetInfo.asset.id,
                    txType = TransactionType.StakeRewards,
                )
            )
        }
    }
}
