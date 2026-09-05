package com.gemwallet.android.features.asset.viewmodels.details.viewmodels

import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.Deeplink
import uniffi.gemstone.GemAssetDetailsServiceInterface
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetChainAssetInfo
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.application.banner.cases.GetActiveBanners
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.model.ChainAssetInfo
import com.gemwallet.android.model.toGem
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModelFactory
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import java.math.BigInteger
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AssetDetailsViewModel @Inject constructor(
    getSession: GetSession,
    savedStateHandle: SavedStateHandle,
    private val getChainAssetInfo: GetChainAssetInfo,
    private val getTransactions: GetTransactions,
    private val assetDetailsService: GemAssetDetailsServiceInterface,
    private val getActiveBanners: GetActiveBanners,
    private val getPriceAlerts: GetPriceAlerts,
    private val assetInfoUIModelFactory: AssetInfoUIModelFactory,
) : ViewModel() {
    private var syncJob: Job? = null

    val session = getSession()

    val isRefreshing = MutableStateFlow(false)

    private val assetId = savedStateHandle.requireAssetId()

    private val chainAssetInfo = getChainAssetInfo(assetId)
        .onStart { restartAssetSync() }
        .filterNotNull()

    private val model = chainAssetInfo.map { chainInfo ->
        val explorerName = assetDetailsService.explorerName(chainInfo.assetInfo.asset.chain.string)
        Model(
            chainAssetInfo = chainInfo,
            explorerName = explorerName,
            updatedAt = System.currentTimeMillis()
        )
    }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val transactions = getTransactions.getTransactions(listOf(TransactionsRequestFilter.Asset(assetId)))
        .map { it.toImmutableList() }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val bannerEvents = chainAssetInfo
        .flatMapLatest { getActiveBanners(it.assetInfo.asset, isGlobal = false) }
        .map { banners -> banners.map { it.event } }

    private val priceAlertsCount = getPriceAlerts(assetId).map { it.size }

    val uiModel = combine(model, session, bannerEvents, priceAlertsCount) { current, session, bannerEvents, priceAlertsCount ->
        val wallet = session?.wallet ?: return@combine null
        current?.let {
            val assetInfo = it.chainAssetInfo.assetInfo
            val asset = assetInfo.asset
            assetInfoUIModelFactory.create(
                chainAssetInfo = it.chainAssetInfo,
                swapPair = assetDetailsService.swapPair(asset.id.toIdentifier(), assetInfo.balance.balance.available.toBigInteger() > BigInteger.ZERO),
                explorerName = it.explorerName,
                explorerAddressUrl = it.chainAssetInfo.assetInfo.owner?.address?.let { address ->
                    assetDetailsService.addressUrl(asset.chain.string, address).link
                },
                explorerTokenUrl = asset.id.tokenId?.let { tokenId ->
                    assetDetailsService.tokenUrl(asset.chain.string, tokenId)?.link
                },
                verificationStatus = assetDetailsService.verificationStatus(asset.toGem(), it.chainAssetInfo.assetInfo.metadata.rankScore)?.toPrimitives(),
                networkDestination = assetDetailsService.networkDestination(asset.id.toIdentifier()),
                shareUrl = assetDetailsService.deeplinkUrl(Deeplink.Asset(assetId = asset.id.toIdentifier())),
                detailsState = assetDetailsService.state(
                    walletType = wallet.type.toGem(),
                    chain = asset.chain.string,
                    metadata = assetInfo.metadata.toGem(),
                    balance = assetInfo.balance.toGem(),
                    bannerEvents = bannerEvents.map { event -> event.toGem() },
                    hasPrice = (assetInfo.price?.price?.price ?: 0.0) != 0.0,
                    priceAlertsCount = priceAlertsCount.toUInt(),
                ),
            )
        }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

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
        val assetInfo = model.value?.chainAssetInfo?.assetInfo ?: return@launch
        assetDetailsService.setAssetPinned(assetInfo.id().toIdentifier(), !assetInfo.metadata.isPinned)
    }

    fun add() = viewModelScope.launch(Dispatchers.IO) {
        val assetInfo = model.value?.chainAssetInfo?.assetInfo ?: return@launch
        assetDetailsService.setAssetsEnabled(listOf(assetInfo.id().toIdentifier()), true)
    }

    private companion object {
        const val TAG = "AssetDetails"
    }

    private data class Model(
        val chainAssetInfo: ChainAssetInfo,
        val updatedAt: Long,
        val explorerName: String,
    )
}
