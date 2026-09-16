package com.gemwallet.android.features.asset.viewmodels.details.viewmodels

import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toGemKey
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.domains.banner.BannerRow
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemPriceAlertToggle
import uniffi.gemstone.GemAssetDetailsInput
import uniffi.gemstone.GemAssetDetailsServiceInterface
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetChainAssetInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.application.banner.cases.GetActiveBanners
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.model.ChainAssetInfo
import com.gemwallet.android.model.Session
import com.gemwallet.android.model.toGem
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModelFactory
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.PriceAlert
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AssetDetailsViewModel @Inject constructor(
    getSession: GetSession,
    savedStateHandle: SavedStateHandle,
    private val getChainAssetInfo: GetChainAssetInfo,
    private val getWalletAssets: GetWalletAssets,
    private val getTransactions: GetTransactions,
    private val assetDetailsService: GemAssetDetailsServiceInterface,
    private val getActiveBanners: GetActiveBanners,
    private val getPriceAlerts: GetPriceAlerts,
    private val assetInfoUIModelFactory: AssetInfoUIModelFactory,
    private val userConfig: UserConfig,
) : ViewModel() {
    private var syncJob: Job? = null

    val session = getSession()

    val isRefreshing = MutableStateFlow(false)

    private val errorState = MutableStateFlow<GemErrorText?>(null)
    val error: StateFlow<GemErrorText?> = errorState.asStateFlow()

    private val assetId = savedStateHandle.requireAssetId()

    private val chainAssetInfo = getChainAssetInfo(assetId)
        .onStart { restartAssetSync() }
        .filterNotNull()
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, storedChainAssetInfo())

    private fun storedChainAssetInfo(): ChainAssetInfo? {
        val stored = getWalletAssets().value
        val assetInfo = stored.firstOrNull { it.asset.id == assetId } ?: return null
        val feeInfo = stored.firstOrNull { it.asset.id == AssetId(assetId.chain) } ?: return null
        return ChainAssetInfo(assetInfo, feeInfo)
    }

    val transactions = getTransactions.getTransactions(listOf(TransactionsRequestFilter.Asset(assetId)))
        .map { it.toImmutableList() }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val banners = chainAssetInfo.filterNotNull()
        .map { it.assetInfo.asset }
        .distinctUntilChanged()
        .flatMapLatest { asset ->
            getActiveBanners(asset).map { banners ->
                banners.map { banner -> BannerRow(banner, assetDetailsService.bannerContent(banner.event.toGem(), banner.asset?.toGem())) }
            }
        }

    private val priceAlerts = getPriceAlerts.assetPriceAlerts(assetId)

    val uiModel = combine(chainAssetInfo, session, banners, priceAlerts, ::uiModel)
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private fun uiModel(
        chainInfo: ChainAssetInfo?,
        session: Session?,
        banners: List<BannerRow>,
        priceAlerts: List<PriceAlert>,
    ): AssetInfoUIModel? {
        val wallet = session?.wallet ?: return null
        val assetInfo = chainInfo?.assetInfo ?: return null
        val details = assetDetailsService.details(
            GemAssetDetailsInput(
                walletType = wallet.type.toGem(),
                asset = assetInfo.asset.toGem(),
                ownerAddress = assetInfo.owner?.address,
                metadata = assetInfo.metadata.toGem(),
                balance = assetInfo.balance.toGem(),
                price = assetInfo.price?.price?.price,
                bannerEvents = banners.map { row -> row.banner.event.toGem() },
                priceAlerts = priceAlerts.map { alert -> alert.toGem() },
            )
        )
        return assetInfoUIModelFactory.create(chainAssetInfo = chainInfo, details = details, banners = banners)
    }

    fun refresh() {
        if (syncJob?.isActive == true) {
            return
        }

        isRefreshing.value = true
        syncPriceAlerts()
        syncJob = viewModelScope.launch(Dispatchers.IO) {
            try {
                syncAssetDetails()
            } finally {
                isRefreshing.value = false
            }
        }
    }

    private fun restartAssetSync() {
        val previousJob = syncJob

        syncPriceAlerts()
        syncJob = viewModelScope.launch(Dispatchers.IO) {
            previousJob?.cancelAndJoin()
            syncAssetDetails()
        }
    }

    private fun syncPriceAlerts() = viewModelScope.launch(Dispatchers.IO) {
        runCatchingCancellable { assetDetailsService.syncPriceAlerts(assetId.toIdentifier()) }
            .onFailure { Log.e(TAG, "price alerts sync failed for ${assetId.toIdentifier()}", it) }
    }

    private suspend fun syncAssetDetails() {
        assetDetailsService
            .refresh(assetId.toIdentifier())
            .forEach { Log.e(TAG, "asset refresh ${it.step} failed: ${it.message}") }
    }

    fun pin() = viewModelScope.launch(Dispatchers.IO) {
        val assetInfo = chainAssetInfo.value?.assetInfo ?: return@launch
        assetDetailsService.setAssetPinned(assetInfo.id().toIdentifier(), !assetInfo.metadata.isPinned)
    }

    fun add() = viewModelScope.launch(Dispatchers.IO) {
        val assetInfo = chainAssetInfo.value?.assetInfo ?: return@launch
        assetDetailsService.setAssetsEnabled(listOf(assetInfo.id().toIdentifier()), true)
    }

    fun togglePriceAlert(assetId: AssetId) = viewModelScope.launch(Dispatchers.IO) {
        val toggled = uiModel.value?.detailsState?.priceAlert?.toggled() ?: return@launch
        runCatchingCancellable { assetDetailsService.setPriceAlert(assetId.toIdentifier(), toggled == GemPriceAlertToggle.ENABLED) }
            .onFailure { errorState.value = it.errorText() }
    }

    fun closeBanner(banner: Banner) = viewModelScope.launch(Dispatchers.IO) {
        runCatchingCancellable { assetDetailsService.closeBanner(banner.toGemKey()) }
            .onFailure { Log.e(TAG, "banner ${banner.event} close failed", it) }
    }

    fun enablePerpetuals() {
        userConfig.setPerpetualEnabled(true)
    }

    fun clearError() = errorState.update { null }

    private companion object {
        const val TAG = "AssetDetails"
    }
}
