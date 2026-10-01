package com.gemwallet.android.features.rewards.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.WalletsQuery
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.rewards.viewmodels.models.RewardsSectionUIModel
import com.gemwallet.android.features.rewards.viewmodels.models.sectionModels
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.navigation.RouteArgument
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemIncomingCode
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemRewardsIntroItem
import uniffi.gemstone.GemRewardsInviteAction
import uniffi.gemstone.GemRewardsPendingReferral
import uniffi.gemstone.GemRewardsRedemption
import uniffi.gemstone.GemRewardsServiceInterface
import uniffi.gemstone.GemRewardsWallet
import uniffi.gemstone.rewardsSession
import javax.inject.Inject

@HiltViewModel
class RewardsViewModel @Inject constructor(
    getSession: GetSession,
    walletsQuery: WalletsQuery,
    private val service: GemRewardsServiceInterface,
    private val savedStateHandle: SavedStateHandle,
    @param:ApplicationContext private val context: Context,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val session = MutableStateFlow(rewardsSession(savedStateHandle.get<String>(RouteArgument.Code.key)))

    private val viewState = session.map { it.viewState(System.currentTimeMillis() / 1000) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, session.value.viewState(System.currentTimeMillis() / 1000))

    val state: StateFlow<GemLoadState> = viewState.map { it.state }
        .stateIn(viewModelScope, SharingStarted.Eagerly, viewState.value.state)

    val isRefreshing: StateFlow<Boolean> = viewState.map { it.isRefreshing }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val wallet: StateFlow<GemRewardsWallet?> = viewState.map { it.wallet }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val incomingCode: StateFlow<GemIncomingCode?> = viewState.map { it.incomingCode }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val rewardsState = viewState.map { it.rewards }
        .stateIn(viewModelScope, SharingStarted.Eagerly, viewState.value.rewards)

    val introItems: StateFlow<List<GemRewardsIntroItem>> = rewardsState.map { it.intro }
        .stateIn(viewModelScope, SharingStarted.Eagerly, rewardsState.value.intro)

    val inviteAction: StateFlow<GemRewardsInviteAction?> = rewardsState.map { it.inviteAction }
        .stateIn(viewModelScope, SharingStarted.Eagerly, rewardsState.value.inviteAction)

    val canUseReferralCode: StateFlow<Boolean> = rewardsState.map { it.canUseReferralCode }
        .stateIn(viewModelScope, SharingStarted.Eagerly, rewardsState.value.canUseReferralCode)

    val pendingReferral: StateFlow<GemRewardsPendingReferral?> = rewardsState.map { it.pendingReferral }
        .stateIn(viewModelScope, SharingStarted.Eagerly, rewardsState.value.pendingReferral)

    val notices: StateFlow<List<GemListRow>> = rewardsState.map { listOfNotNull(it.errorNotice, it.statusNotice) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val inviteDescription: StateFlow<String> = rewardsState.map { it.inviteDescription.string(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, rewardsState.value.inviteDescription.string(context))

    val shareText: StateFlow<String?> = rewardsState.map { it.shareText?.string(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, rewardsState.value.shareText?.string(context))

    val sections: StateFlow<List<RewardsSectionUIModel>> = rewardsState.map { it.sectionModels(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val redemptions: StateFlow<List<GemRewardsRedemption>> = rewardsState.map { state -> state.redemptions }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val referralLink = rewardsState.map { it.referralLink }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    init {
        viewModelScope.launch {
            combine(walletsQuery(), getSession().filterNotNull()) { wallets, current -> wallets.map { it.toGem() } to current.wallet.id.id }
                .collect { (wallets, current) -> session.update { it.onWallets(wallets, current) } }
        }
        viewModelScope.launch {
            session.distinctUntilChangedBy { it.wallet?.id to it.needsLoad() }
                .collectLatest { current -> current.wallet?.takeIf { current.needsLoad() }?.let { load(it.id) } }
        }
    }

    fun setWallet(walletId: String) {
        session.update { it.onSelectWallet(walletId) }
    }

    fun sync() {
        session.update { it.onRefreshing() }
    }

    private suspend fun load(walletId: String) {
        val result = withContext(ioDispatcher) { service.refresh(walletId) }
        session.update { it.onResult(result) }
    }

    fun createReferral(username: String, callback: (Throwable?) -> Unit) {
        val walletId = session.value.wallet?.id ?: return
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { service.createReferral(walletId, username) }
                .onSuccess { rewards -> session.update { it.onRewards(walletId, rewards) } }
                .report(callback)
        }
    }

    fun useCode(code: String, callback: (Throwable?) -> Unit) {
        val walletId = session.value.wallet?.id ?: return
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { service.useReferralCode(walletId, code) }
                .onSuccess { rewards -> session.update { it.onRewards(walletId, rewards) } }
                .report(callback)
        }
    }

    fun redeem(redemption: GemRewardsRedemption, callback: (Throwable?) -> Unit) {
        val walletId = session.value.wallet?.id ?: return
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { service.redeem(walletId, redemption.id) }
                .onSuccess { sync() }
                .report(callback)
        }
    }

    private suspend fun <T> Result<T>.report(callback: (Throwable?) -> Unit) = withContext(Dispatchers.Main) {
        callback(exceptionOrNull())
    }

    fun onCodeHandled() {
        session.update { it.onCodeHandled() }
        savedStateHandle[RouteArgument.Code.key] = null
    }
}
