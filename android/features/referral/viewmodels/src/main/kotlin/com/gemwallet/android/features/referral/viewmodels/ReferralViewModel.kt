package com.gemwallet.android.features.referral.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.referral.viewmodels.models.IncomingCodeUIModel
import com.gemwallet.android.features.referral.viewmodels.models.RewardRedemptionUIModel
import com.gemwallet.android.features.referral.viewmodels.models.RewardsSectionUIModel
import com.gemwallet.android.features.referral.viewmodels.models.sectionModels
import com.gemwallet.android.features.referral.viewmodels.models.uiModel
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Wallet
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemIncomingCode
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemRewardsAction
import uniffi.gemstone.GemRewardsRedemption
import uniffi.gemstone.GemRewardsServiceInterface
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.incomingReferralCode
import uniffi.gemstone.loadError
import uniffi.gemstone.rewardsSession
import uniffi.gemstone.walletRows
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ReferralViewModel @Inject constructor(
    getSession: GetSession,
    getWallets: GetWallets,
    private val service: GemRewardsServiceInterface,
    private val savedStateHandle: SavedStateHandle,
    @param:ApplicationContext private val context: Context,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    val referralCode = savedStateHandle.getStateFlow<String?>(RouteArgument.Code.key, null)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val currentWallet = MutableStateFlow<Wallet?>(null)
    private val session = MutableStateFlow(rewardsSession())

    private val viewState = session.map { it.viewState(System.currentTimeMillis() / 1000) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, session.value.viewState(System.currentTimeMillis() / 1000))

    val isLoading: StateFlow<Boolean> = viewState.map { it.state == GemLoadState.Loading }
        .stateIn(viewModelScope, SharingStarted.Eagerly, true)

    val isRefreshing: StateFlow<Boolean> = viewState.map { it.isRefreshing }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val loadError: StateFlow<GemServiceException?> = viewState.map { loadError(it.state, hasRows = false) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val rewardsState = viewState.map { it.rewards }
        .stateIn(viewModelScope, SharingStarted.Eagerly, viewState.value.rewards)

    val actions: StateFlow<List<GemRewardsAction>> = rewardsState.map { it.actions }
        .stateIn(viewModelScope, SharingStarted.Eagerly, rewardsState.value.actions)

    val notices: StateFlow<List<GemListRow>> = rewardsState.map { listOfNotNull(it.errorNotice, it.statusNotice) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val inviteRewardPoints: StateFlow<String> = rewardsState.map { it.inviteRewardPoints.text() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, rewardsState.value.inviteRewardPoints.text())

    val sections: StateFlow<List<RewardsSectionUIModel>> = rewardsState.map { it.sectionModels(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val redemptions: StateFlow<List<RewardRedemptionUIModel>> = rewardsState.map { state -> state.redemptions.map { it.uiModel(context) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val referralLink = rewardsState.mapLatest { it.referralLink }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val availableWallets = getWallets().mapLatest { wallets -> service.wallets(wallets.map { it.toGem() }).map { it.toPrimitives() } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val availableWalletRows = availableWallets.mapLatest { wallets -> walletRows(wallets.map { it.toGem() }) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val incomingCode: StateFlow<IncomingCodeUIModel> = combine(referralCode, availableWallets) { code, wallets ->
        incomingReferralCode(code, wallets.map { it.toGem() }).uiModel()
    }.stateIn(viewModelScope, SharingStarted.Eagerly, IncomingCodeUIModel())

    private val selectedWallet = getSession()
        .filterNotNull()
        .combine(availableWallets) { current, wallets -> service.selectedWallet(current?.wallet?.toGem(), wallets.map { it.toGem() })?.toPrimitives() }
        .onEach { wallet ->
            currentWallet.update {
                if (it?.id == null || it.id == wallet?.id) {
                    wallet
                } else {
                    it
                }
            }
        }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val referralWallet = currentWallet.filterNotNull()
        .distinctUntilChangedBy { it.id.id }
        .onEach { wallet ->
            session.update { it.onSelectWallet(wallet.id.id) }
            sync(wallet)
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun setWallet(walletId: String) {
        val wallet = availableWallets.value.firstOrNull { it.id.id == walletId } ?: return
        currentWallet.update { wallet }
    }

    fun sync() {
        val wallet = referralWallet.value ?: return
        session.update { it.onRefreshing() }
        sync(wallet)
    }

    private fun sync(wallet: Wallet) = viewModelScope.launch(ioDispatcher) {
        val result = service.refresh(wallet.id.id)
        session.update { it.onResult(result) }
    }

    fun createReferral(username: String, callback: (Throwable?) -> Unit) = viewModelScope.launch(ioDispatcher) {
        val wallet = currentWallet.value ?: return@launch
        runCatchingCancellable { service.createReferral(wallet.toGem(), username) }
            .onSuccess { rewards -> session.update { it.onRewards(rewards) } }
            .report(callback)
    }

    fun useCode(code: String, callback: (Throwable?) -> Unit) = viewModelScope.launch(ioDispatcher) {
        val wallet = currentWallet.value ?: return@launch
        runCatchingCancellable { service.useReferralCode(wallet.toGem(), code) }
            .onSuccess { rewards -> session.update { it.onRewards(rewards) } }
            .report(callback)
    }

    fun redeem(redemption: GemRewardsRedemption, callback: (Throwable?) -> Unit) {
        val wallet = currentWallet.value ?: return
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { service.redeem(wallet.toGem(), redemption.id) }
                .onSuccess { sync() }
                .report(callback)
        }
    }

    private suspend fun <T> Result<T>.report(callback: (Throwable?) -> Unit) = withContext(Dispatchers.Main) {
        callback(exceptionOrNull())
    }

    fun cancelCode() {
        savedStateHandle[RouteArgument.Code.key] = null
    }
}
