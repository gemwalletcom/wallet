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
import com.gemwallet.android.features.referral.viewmodels.models.RewardRedemptionUIModel
import com.gemwallet.android.features.referral.viewmodels.models.infoRows
import com.gemwallet.android.features.referral.viewmodels.models.uiModel
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
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemIncomingCode
import uniffi.gemstone.GemRewardsRedemption
import uniffi.gemstone.GemRewardsServiceInterface
import uniffi.gemstone.Rewards
import uniffi.gemstone.incomingReferralCode
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
    private val rewards = MutableStateFlow<Rewards?>(null)
    val inSync = MutableStateFlow(SyncType.Init)

    val uiState = rewards.mapLatest { service.state(it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, service.state(null))

    val infoRows: StateFlow<List<ListItemModel>> = uiState.map { it.infoRows(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val redemptions: StateFlow<List<RewardRedemptionUIModel>> = uiState.map { state -> state.redemptions.mapNotNull { it.uiModel(context) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val referralLink = uiState.mapLatest { it.referralLink }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val availableWallets = getWallets().mapLatest { wallets -> service.wallets(wallets.map { it.toGem() }).map { it.toPrimitives() } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val availableWalletRows = availableWallets.mapLatest { wallets -> walletRows(wallets.map { it.toGem() }) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val incomingCode: StateFlow<GemIncomingCode?> = combine(referralCode, availableWallets) { code, wallets ->
        incomingReferralCode(code, wallets.map { it.toGem() })
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val session = getSession()
        .filterNotNull()
        .combine(availableWallets) { session, wallets -> service.selectedWallet(session?.wallet?.toGem(), wallets.map { it.toGem() })?.toPrimitives() }
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
        .onEach { sync(it, SyncType.Init) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun setWallet(walletId: String) {
        val wallet = availableWallets.value.firstOrNull { it.id.id == walletId } ?: return
        currentWallet.update { wallet }
    }

    fun sync() {
        sync(referralWallet.value ?: return, SyncType.Refresh)
    }

    private fun sync(wallet: Wallet, type: SyncType) = viewModelScope.launch(ioDispatcher) {
        inSync.update { type }
        val rewards = try {
            runCatchingCancellable { service.getRewards(wallet.id.id) }.getOrNull()
        } finally {
            inSync.update { SyncType.None }
        }
        this@ReferralViewModel.rewards.update { rewards }
    }

    fun createReferral(username: String, callback: (Exception?) -> Unit) = viewModelScope.launch(ioDispatcher) {
        val rewards = try {
            val wallet = currentWallet.value ?: return@launch
            val response = service.createReferral(wallet.toGem(), username)
            withContext(Dispatchers.Main) {
                callback(null)
            }
            response
        } catch (err: Exception) {
            withContext(Dispatchers.Main) {
                callback(err)
            }
            null
        }
        this@ReferralViewModel.rewards.update { rewards }
    }

    fun useCode(code: String, callback: (Exception?) -> Unit) = viewModelScope.launch(ioDispatcher) {
        try {
            val wallet = currentWallet.value ?: return@launch
            val rewards = service.useReferralCode(wallet.toGem(), code)
            this@ReferralViewModel.rewards.update { rewards }
            withContext(Dispatchers.Main) {
                callback(null)
            }
        } catch (err: Exception) {
            withContext(Dispatchers.Main) {
                callback(err)
            }
        }
    }

    fun redeem(redemption: GemRewardsRedemption, callback: (Throwable?) -> Unit) {
        val wallet = currentWallet.value ?: return
        viewModelScope.launch(ioDispatcher) {
            try {
                service.redeem(wallet.toGem(), redemption.option.id)
                sync()
                withContext(Dispatchers.Main) {
                    callback(null)
                }
            } catch (err: Throwable) {
                withContext(Dispatchers.Main) {
                    callback(err)
                }
            }
        }
    }

    fun cancelCode() {
        savedStateHandle[RouteArgument.Code.key] = null
    }
}
