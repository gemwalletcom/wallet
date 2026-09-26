package com.gemwallet.android.features.assets.viewmodels.asset

import android.content.Context
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.connection.cases.ObserveRefreshInterval
import com.gemwallet.android.application.preferences.cases.ObservablePreferences
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.data.services.store.queries.BannersQuery
import com.gemwallet.android.data.services.store.queries.ChainAssetQuery
import com.gemwallet.android.data.services.store.queries.PriceAlertsQuery
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.assets.viewmodels.asset.models.AssetUIState
import com.gemwallet.android.features.assets.viewmodels.asset.models.AssetUIStateFactory
import com.gemwallet.android.model.Session
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.assetAddedToast
import com.gemwallet.android.ui.components.screen.assetPinnedToast
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.localization.toastRes
import com.gemwallet.android.ui.models.ToastEmitter
import com.gemwallet.android.ui.models.ToastEmitterImpl
import com.gemwallet.android.ui.models.ToastMessage
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.ChainAssetData
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.TransactionsFilter
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancelAndJoin
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
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAssetDetailsInput
import uniffi.gemstone.GemAssetDetailsServiceInterface
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemPriceAlertToggle
import uniffi.gemstone.GemRefreshKind
import uniffi.gemstone.loadError
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AssetViewModel @Inject constructor(
    getSession: GetSession,
    savedStateHandle: SavedStateHandle,
    getCurrentWalletId: GetCurrentWalletId,
    chainAssetQuery: ChainAssetQuery,
    private val getWalletAssets: GetWalletAssets,
    private val getTransactions: GetTransactions,
    private val assetDetailsService: GemAssetDetailsServiceInterface,
    private val bannersQuery: BannersQuery,
    private val priceAlertsQuery: PriceAlertsQuery,
    private val assetUIStateFactory: AssetUIStateFactory,
    private val preferences: ObservablePreferences,
    private val observeRefreshInterval: ObserveRefreshInterval,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel(),
    ToastEmitter by ToastEmitterImpl() {

    val refreshIntervalMillis: StateFlow<Long> = observeRefreshInterval.refreshIntervalMillis(GemRefreshKind.WALLET)
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)

    private var syncJob: Job? = null

    val session = getSession()

    val isRefreshing = MutableStateFlow(false)

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

    private val assetId = savedStateHandle.requireAssetId()

    private val transactionsFilter = TransactionsFilter(assetId = assetId, chains = emptyList(), transactionTypes = emptyList(), states = emptyList())

    val transactions = getTransactions.getTransactions(transactionsFilter, GemConstants.transactionsListLimit)
        .map { it.toImmutableList() }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, getTransactions.stored(transactionsFilter, GemConstants.transactionsListLimit).toImmutableList())

    private val transactionsState = MutableStateFlow<GemLoadState>(GemLoadState.Loading)

    private val chainAssetInfo = getCurrentWalletId().flatMapLatest { walletId -> chainAssetQuery(walletId.id, assetId) }
        .onStart { restartAssetSync() }
        .filterNotNull()
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, storedChainAssetInfo())

    private fun storedChainAssetInfo(): ChainAssetData? {
        val stored = getWalletAssets().value
        val assetInfo = stored.firstOrNull { it.asset.id == assetId } ?: return null
        val feeInfo = stored.firstOrNull { it.asset.id == AssetId(assetId.chain) } ?: return null
        return ChainAssetData(assetInfo, feeInfo)
    }

    val transactionsErrorRow: StateFlow<GemListRow?> = combine(transactionsState, transactions) { state, items ->
        loadError(state, items.isNotEmpty())?.let { GemListRow.Error(it) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val banners = chainAssetInfo.filterNotNull()
        .map { it.assetData.asset }
        .distinctUntilChanged()
        .flatMapLatest { asset ->
            session.flatMapLatest { bannersQuery(it?.wallet?.id?.id, asset.id) }
        }

    private val priceAlerts = priceAlertsQuery(assetId).map { alerts -> alerts.map { it.priceAlert } }

    val uiModel = combine(chainAssetInfo, session, banners, priceAlerts, ::uiModel)
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private fun uiModel(chainInfo: ChainAssetData?, session: Session?, banners: List<Banner>, priceAlerts: List<PriceAlert>): AssetUIState? {
        session ?: return null
        val wallet = session.wallet
        val assetData = chainInfo?.assetData ?: return null
        val details = assetDetailsService.details(
            GemAssetDetailsInput(
                wallet = wallet.toGem(),
                assetData = assetData.copy(priceAlerts = priceAlerts).toGem(),
                currency = session.currency.toGem(),
                banners = banners.map { banner -> banner.toGem() },
                feeBalanceMetadata = chainInfo.feeAssetData.balance.metadata?.toGem(),
            ),
        )
        return assetUIStateFactory.create(chainAssetInfo = chainInfo, details = details)
    }

    fun refresh() {
        if (syncJob?.isActive == true) {
            return
        }

        isRefreshing.value = true
        syncJob = viewModelScope.launch(ioDispatcher) {
            try {
                syncAssetDetails()
            } finally {
                isRefreshing.value = false
            }
        }
    }

    private fun restartAssetSync() {
        val previousJob = syncJob

        syncJob = viewModelScope.launch(ioDispatcher) {
            previousJob?.cancelAndJoin()
            syncAssetDetails()
        }
    }

    private suspend fun syncAssetDetails() {
        val refresh = assetDetailsService.refresh(assetId.toIdentifier(), transactions.value.isNotEmpty())
        transactionsState.value = refresh.transactions
        refresh.failures.forEach { Log.e(TAG, "asset refresh ${it.step} failed: ${it.message}") }
    }

    fun pin() = viewModelScope.launch(ioDispatcher) {
        val assetData = chainAssetInfo.value?.assetData ?: return@launch
        runCatchingCancellable { assetDetailsService.setAssetPinned(assetData.asset.id.toIdentifier(), !assetData.metadata.isPinned) }
            .onSuccess { emitToast(assetPinnedToast(context, assetData.asset.name, !assetData.metadata.isPinned)) }
            .onFailure { Log.e(TAG, "pinning ${assetData.asset.id.toIdentifier()} failed", it) }
    }

    fun add() = viewModelScope.launch(ioDispatcher) {
        val assetData = chainAssetInfo.value?.assetData ?: return@launch
        runCatchingCancellable { assetDetailsService.setAssetsEnabled(listOf(assetData.asset.id.toIdentifier()), true) }
            .onSuccess { emitToast(assetAddedToast(context)) }
            .onFailure { emitToast(ToastMessage(it.errorText().text(context), R.drawable.ic_error)) }
    }

    fun togglePriceAlert(assetId: AssetId) = viewModelScope.launch(ioDispatcher) {
        val current = uiModel.value?.details?.state?.priceAlert ?: return@launch
        val name = chainAssetInfo.value?.assetData?.asset?.name.orEmpty()
        runCatchingCancellable { assetDetailsService.setPriceAlert(assetId.toIdentifier(), current.toggled() == GemPriceAlertToggle.ENABLED) }
            .onSuccess { emitToast(ToastMessage(context.getString(current.toastRes(), name), R.drawable.ic_notifications)) }
            .onFailure { errorState.value = it.errorText().text(context) }
    }

    fun closeBanner(key: GemBannerKey) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { assetDetailsService.closeBanner(key) }
            .onFailure { errorState.value = it.errorText().text(context) }
    }

    fun enablePerpetuals() {
        preferences.setPerpetualEnabled(true)
    }

    fun clearError() = errorState.update { null }

    private companion object {
        const val TAG = "AssetDetails"
    }
}
