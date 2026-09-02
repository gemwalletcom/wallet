package com.gemwallet.android.features.asset.viewmodels.details.viewmodels

import android.util.Log
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemAssetDetailsService
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetChainAssetInfo
import com.gemwallet.android.application.pricealerts.cases.SyncAssetPriceAlerts
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.application.banner.cases.HasMultiSign
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.model.ChainAssetInfo
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModelFactory
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet
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
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AssetDetailsViewModel @Inject constructor(
    getSession: GetSession,
    savedStateHandle: SavedStateHandle,
    private val getChainAssetInfo: GetChainAssetInfo,
    private val getTransactions: GetTransactions,
    private val syncAssetPriceAlerts: SyncAssetPriceAlerts,
    private val assetDetailsService: GemAssetDetailsService,
    private val getCurrentCurrency: GetCurrentCurrency,
    private val hasMultiSign: HasMultiSign,
    private val assetInfoUIModelFactory: AssetInfoUIModelFactory,
) : ViewModel() {
    private var syncJob: Job? = null

    val session = getSession()

    val isRefreshing = MutableStateFlow(false)

    private val assetId = savedStateHandle.requireAssetId()

    private val chainAssetInfo = getChainAssetInfo(assetId)
        .onStart {
            val wallet = session.value?.wallet ?: return@onStart
            restartAssetSync(wallet)
        }
        .filterNotNull()

    val isOperationEnabled = session.filterNotNull().flatMapLatest {
        hasMultiSign.hasMultiSign(it.wallet).mapLatest { !it }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, true)

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

    val uiModel = combine(model, session) { current, session ->
        val wallet = session?.wallet ?: return@combine null
        current?.let {
            val asset = it.chainAssetInfo.assetInfo.asset
            assetInfoUIModelFactory.create(
                chainAssetInfo = it.chainAssetInfo,
                explorerName = it.explorerName,
                walletType = wallet.type,
                explorerAddressUrl = it.chainAssetInfo.assetInfo.owner?.address?.let { address ->
                    assetDetailsService.addressUrl(asset.chain.string, address).link
                },
                explorerTokenUrl = asset.id.tokenId?.let { tokenId ->
                    assetDetailsService.tokenUrl(asset.chain.string, tokenId)?.link
                },
            )
        }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun refresh() {
        val wallet = session.value?.wallet ?: return
        if (syncJob?.isActive == true) {
            return
        }

        isRefreshing.value = true
        syncPriceAlerts()
        syncJob = viewModelScope.launch(Dispatchers.IO) {
            try {
                syncAssetDetails(wallet)
            } finally {
                isRefreshing.value = false
            }
        }
    }

    private fun restartAssetSync(wallet: Wallet) {
        val previousJob = syncJob

        syncPriceAlerts()
        syncJob = viewModelScope.launch(Dispatchers.IO) {
            previousJob?.cancelAndJoin()
            syncAssetDetails(wallet)
        }
    }

    private fun syncPriceAlerts() = viewModelScope.launch(Dispatchers.IO) {
        syncAssetPriceAlerts(assetId)
    }

    private suspend fun syncAssetDetails(wallet: Wallet) {
        wallet.getAccount(assetId) ?: return
        assetDetailsService
            .refresh(wallet.id.id, assetId.toIdentifier(), getCurrentCurrency.getCurrentCurrency().toJson())
            .forEach { Log.e(TAG, "asset refresh ${it.step} failed: ${it.message}") }
    }

    fun pin() = viewModelScope.launch(Dispatchers.IO) {
        val wallet = session.value?.wallet ?: return@launch
        val assetInfo = model.value?.chainAssetInfo?.assetInfo ?: return@launch
        val assetId = assetInfo.id()
        wallet.getAccount(assetId) ?: return@launch
        assetDetailsService.setAssetPinned(wallet.id.id, assetId.toIdentifier(), !assetInfo.metadata.isPinned)
    }

    fun add() = viewModelScope.launch(Dispatchers.IO) {
        val session = session.value ?: return@launch
        val assetInfo = model.value?.chainAssetInfo?.assetInfo ?: return@launch

        add(session.wallet, assetInfo.id())
    }

    private suspend fun add(wallet: Wallet, assetId: AssetId) {
        wallet.getAccount(assetId) ?: return
        assetDetailsService.setAssetsEnabled(wallet.id.id, listOf(assetId.toIdentifier()), true)
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
