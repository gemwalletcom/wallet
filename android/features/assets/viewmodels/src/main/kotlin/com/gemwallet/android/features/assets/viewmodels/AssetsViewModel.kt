package com.gemwallet.android.features.assets.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.coordinators.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.coordinators.GetWalletSummary
import com.gemwallet.android.application.wallet_import.coordinators.GetImportWalletState
import com.gemwallet.android.application.wallet_import.values.ImportWalletState
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.model.SyncState
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.WalletSource
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AssetsViewModel @Inject constructor(
    sessionRepository: SessionRepository,
    private val assetsRepository: AssetsRepository,
    private val userConfig: UserConfig,
    private val getImportWalletState: GetImportWalletState,
    getActiveAssetsInfo: GetActiveAssetsInfo,
    getWalletSummary: GetWalletSummary,
) : ViewModel() {

    private val session = sessionRepository.session()

    private data class AssetGroups(
        val pinned: List<AssetInfoDataAggregate> = emptyList(),
        val unpinned: List<AssetInfoDataAggregate> = emptyList(),
    )

    val importInProgress = session
        .filterNotNull()
        .flatMapLatest { session ->
            getImportWalletState
                .getImportState(session.wallet.id)
                .mapLatest { it == ImportWalletState.Importing }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val refreshingState = MutableStateFlow<RefreshingState>(RefreshingState.OnOpen)
    val screenState = refreshingState.map { refreshingState ->
            when (refreshingState) {
                RefreshingState.OnOpen -> SyncState.Idle
                RefreshingState.OnForce -> SyncState.InSync
            }
        }
        .flatMapLatest { state ->
            flow {
                emit(state)
                delay(1000)
                emit(SyncState.Idle)
            }
        }
        .map { it == SyncState.InSync }
        .stateIn(viewModelScope, SharingStarted.Eagerly, true)

    private val isHideBalances = userConfig.isHideBalances()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    private val assetGroups = getActiveAssetsInfo.getAssetsInfo(isHideBalances)
        .map { items ->
            val (pinned, unpinned) = items.partition { it.pinned }
            AssetGroups(
                pinned = pinned,
                unpinned = unpinned,
            )
        }
        .flowOn(Dispatchers.Default)
        .stateIn(viewModelScope, SharingStarted.Eagerly, AssetGroups())

    private val isWalletEmpty = assetGroups
        .map { groups ->
            groups.pinned.all { it.isZeroBalance }
                && groups.unpinned.all { it.isZeroBalance }
        }
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val pinnedAssets = assetGroups
        .map { it.pinned }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val unpinnedAssets = assetGroups
        .map { it.unpinned }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val walletSummary = getWalletSummary.getWalletSummary()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val showWelcomeBanner = session
        .filterNotNull()
        .flatMapLatest { session ->
            combine(isWalletEmpty, userConfig.isWelcomeBannerHidden(session.wallet.id)) { isWalletEmpty, isWelcomeBannerHidden ->
                val created = session.wallet.source == WalletSource.Create
                isWalletEmpty && created && !isWelcomeBannerHidden
            }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    fun onRefresh() {
        session.value?.let { session ->
            refreshingState.update { RefreshingState.OnForce }
            updateAssetData()
        }
    }

    private fun updateAssetData() = viewModelScope.launch(Dispatchers.IO) {
        assetsRepository.sync()
    }

    fun hideAsset(assetId: AssetId) = viewModelScope.launch {
        val session = session.value ?: return@launch
        val account = session.wallet.getAccount(assetId.chain) ?: return@launch
        assetsRepository.switchVisibility(session.wallet.id, account, assetId, false)
    }

    fun togglePin(assetId: AssetId) = viewModelScope.launch {
        val session = session.value ?: return@launch
        assetsRepository.togglePin(session.wallet.id, assetId)
    }

    fun hideBalances() = viewModelScope.launch {
        userConfig.hideBalances()
    }

    fun onHideWelcomeBanner() = viewModelScope.launch {
        userConfig.hideWelcomeBanner(session.value?.wallet?.id ?: return@launch)
    }

    enum class RefreshingState {
        OnOpen,
        OnForce,
    }
}
