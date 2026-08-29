package com.gemwallet.android.data.repositories.assets

import android.util.Log
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.gemstone.GemstoneAssetStore
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.entities.toAssetInfoModel
import com.gemwallet.android.data.service.store.database.entities.toAssetLinksModel
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import com.gemwallet.android.serializer.toJson
import javax.inject.Inject
import javax.inject.Singleton
import com.gemwallet.android.ext.available
import com.gemwallet.android.ext.runCatchingCancellable
import uniffi.gemstone.GemBalanceService

private const val TAG = "AssetsRepository"

@OptIn(ExperimentalCoroutinesApi::class)
@Singleton
class AssetsRepository @Inject constructor(
    private val assetsDao: AssetsDao,
    private val assetStore: GemstoneAssetStore,
    private val sessionRepository: SessionRepository,
    private val searchTokensCase: SearchTokensCase,
    private val balanceService: GemBalanceService,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) {

    private fun currentWalletId(): Flow<String> = sessionRepository.currentWalletId()

    suspend fun sync() {
        getAssetsInfo().firstOrNull()?.refreshBalances()?.awaitAll()
    }

    private val assetsInfo: Flow<List<AssetInfo>> = currentWalletId()
        .flatMapLatest { walletId -> assetStore.observeAssetsInfo(walletId) }
        .shareIn(scope, SharingStarted.Eagerly, replay = 1)

    fun getAssetsInfo(): Flow<List<AssetInfo>> = assetsInfo

    fun getAssetsInfo(walletId: WalletId): Flow<List<AssetInfo>> = assetStore.observeAssetsInfo(walletId.id).flowOn(Dispatchers.IO)

    fun getAssetsInfo(assetsId: List<AssetId>): Flow<List<AssetInfo>> = currentWalletId()
        .flatMapLatest { walletId -> assetStore.observeAssetsInfo(walletId, assetsId.map { it.toIdentifier() }) }
        .flowOn(Dispatchers.IO)

    fun getAssetsInfoByChain(chain: Chain): Flow<List<AssetInfo>> = currentWalletId()
        .flatMapLatest { walletId -> assetStore.observeAssetsInfoByChain(walletId, chain) }
        .flowOn(Dispatchers.IO)

    fun getHiddenAssetsInfoByChain(chain: Chain): Flow<List<AssetInfo>> = currentWalletId()
        .flatMapLatest { walletId -> assetStore.observeHiddenAssetsInfoByChain(walletId, chain) }
        .flowOn(Dispatchers.IO)

    fun getAssetInfo(assetId: AssetId): Flow<AssetInfo?> {
        return currentWalletId()
            .flatMapLatest { walletId -> assetStore.observeAssetInfo(walletId, assetId) }
            .flowOn(Dispatchers.IO)
    }

    fun asset(assetId: AssetId): Flow<Asset?> = assetStore.observeAsset(assetId).flowOn(Dispatchers.IO)

    fun getToken(assetId: AssetId): Flow<Asset?> = currentWalletId()
        .flatMapLatest { walletId -> assetStore.observeTokenInfo(walletId, assetId) }
        .map { it?.asset }
        .flowOn(Dispatchers.IO)

    fun getTokenInfo(assetId: AssetId): Flow<AssetInfo?> {
        return currentWalletId().flatMapLatest { walletId ->
            assetStore.observeAssetInfo(walletId, assetId).flatMapLatest { assetInfo ->
                assetInfo?.let { flowOf(it) } ?: assetStore.observeTokenInfo(walletId, assetId)
            }
        }
        .flowOn(Dispatchers.IO)
    }

    fun getTokensInfo(assetsId: List<String>): Flow<List<AssetInfo>> = currentWalletId()
        .flatMapLatest { walletId -> assetStore.observeAssetsInfoByAllWallets(walletId, assetsId) }

    suspend fun getWidgetTokens(currency: Currency): List<AssetInfo> = withContext(Dispatchers.IO) {
        val widgetAssetIds = listOf(AssetId(Chain.Bitcoin), AssetId(Chain.Ethereum), AssetId(Chain.Solana))

        runCatching { searchTokensCase.search(widgetAssetIds, currency) }
        getTokensInfo(widgetAssetIds.map { it.toIdentifier() }).firstOrNull() ?: emptyList()
    }

    suspend fun searchToken(assetId: AssetId, currency: Currency): Boolean {
        return searchTokensCase.search(assetId, currency)
    }

    /**
     * Check and add new coins and active tokens
     * */
    suspend fun updateBalances(vararg tokens: AssetId) {
        getAssetsInfo(tokens.toList()).firstOrNull()?.refreshBalances()?.awaitAll()
    }

    suspend fun updateBalances(assetInfos: List<AssetInfo>) {
        assetInfos.refreshBalances().awaitAll()
    }

    fun getAssetLinks(id: AssetId): Flow<List<AssetLink>> = assetStore.observeAssetLinks(id).flowOn(Dispatchers.IO)

    fun getAssetMarket(id: AssetId): Flow<AssetMarket?> = assetStore.observeAssetMarket(id).flowOn(Dispatchers.IO)

    private suspend fun List<AssetInfo>.refreshBalances(): List<Deferred<Unit>> = withContext(Dispatchers.IO) {
        groupBy { it.walletId }
            .mapNotNull { (walletId, assetInfos) ->
                walletId ?: return@mapNotNull null
                async {
                    runCatchingCancellable { balanceService.update(walletId.id, assetInfos.map { it.asset.id.toIdentifier() }) }
                        .onFailure { Log.e(TAG, "balances update failed for ${walletId.id}", it) }
                    Unit
                }
            }
    }

    private companion object {
        const val TAG = "AssetsRepository"
    }

}
