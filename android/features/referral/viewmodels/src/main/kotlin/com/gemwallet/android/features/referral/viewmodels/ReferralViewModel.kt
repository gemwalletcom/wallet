package com.gemwallet.android.features.referral.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.domains.referral.values.ReferralError
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.RewardRedemptionOption
import com.wallet.core.primitives.Rewards
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemRewardsServiceInterface
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ReferralViewModel @Inject constructor(
    getSession: GetSession,
    getWallets: GetWallets,
    private val service: GemRewardsServiceInterface,
    private val savedStateHandle: SavedStateHandle,
) : ViewModel() {

    val referralCode = savedStateHandle.getStateFlow<String?>(RouteArgument.Code.key, null)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val currentWallet = MutableStateFlow<Wallet?>(null)
    val rewards = MutableStateFlow<Rewards?>(null)
    val inSync = MutableStateFlow(SyncType.Init)

    val uiState = rewards.mapLatest { RewardsUIState.from(it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, RewardsUIState.from(null))

    val referralLink = rewards.mapLatest { it?.code?.let(service::referralLink) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val availableWallets = getWallets().mapLatest { service.wallets().map { it.decodeJson<Wallet>() } }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val session = getSession()
        .filterNotNull()
        .combine(availableWallets) { _, _ -> service.selectedWallet()?.decodeJson<Wallet>() }
        .flowOn(Dispatchers.IO)
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

    fun setWallet(wallet: Wallet) {
        currentWallet.update { wallet }
    }

    fun sync() {
        sync(referralWallet.value ?: return, SyncType.Refresh)
    }

    private fun sync(wallet: Wallet, type: SyncType) = viewModelScope.launch(Dispatchers.IO) {
        inSync.update { type }
        val rewards = try {
            service.getRewards(wallet.id.id).decodeJson<Rewards>().takeIf { it.code != null }
        } catch (_: Exception) {
            null
        } finally {
            inSync.update { SyncType.None }
        }
        this@ReferralViewModel.rewards.update { rewards }
    }

    fun createReferral(username: String, callback: (Exception?) -> Unit) = viewModelScope.launch(Dispatchers.IO) {
        val rewards = try {
            val wallet = currentWallet.value ?: return@launch
            val response = service.createReferral(wallet.toJson(), username).decodeJson<Rewards>()
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

    fun useCode(code: String, callback: (Exception?) -> Unit) = viewModelScope.launch(Dispatchers.IO) {
        try {
            val wallet = currentWallet.value ?: return@launch
            service.useReferralCode(wallet.toJson(), code)
            withContext(Dispatchers.Main) {
                callback(null)
            }
        } catch (err: Exception) {
            withContext(Dispatchers.Main) {
                callback(err)
            }
        }
    }

    fun redeem(option: RewardRedemptionOption, callback: (Throwable?) -> Unit) {
        val wallet = currentWallet.value ?: return
        val rewards = rewards.value ?: return
        viewModelScope.launch(Dispatchers.IO) {
            try {
                if (rewards.points < option.points) throw ReferralError.InsufficientPoints
                service.redeem(wallet.toJson(), option.id)
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
