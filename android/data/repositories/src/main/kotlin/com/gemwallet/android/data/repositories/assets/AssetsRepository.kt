package com.gemwallet.android.data.repositories.assets

import android.util.Log
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.entities.DbAsset
import com.gemwallet.android.data.service.store.database.entities.DbAssetBasicUpdate
import com.gemwallet.android.data.service.store.database.entities.toAssetInfoModel
import com.gemwallet.android.data.service.store.database.entities.toAssetLinksModel
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.service.store.database.entities.toUpdateRecord
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.defaultBasic
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
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import com.gemwallet.android.serializer.toJson
import javax.inject.Inject
import javax.inject.Singleton
import uniffi.gemstone.GemStreamSubscriptionService
import com.gemwallet.android.ext.available
import com.gemwallet.android.ext.runCatchingCancellable
import uniffi.gemstone.GemBalanceService

private const val TAG = "AssetsRepository"

@OptIn(ExperimentalCoroutinesApi::class)
@Singleton
class AssetsRepository @Inject constructor(
    private val assetsDao: AssetsDao,
    private val sessionRepository: SessionRepository,
    private val searchTokensCase: SearchTokensCase,
    private val streamSubscriptionService: GemStreamSubscriptionService,
    private val balanceService: GemBalanceService,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) {

    private fun currentWalletId(): Flow<String> = sessionRepository.currentWalletId()

    suspend fun sync() {
        getAssetsInfo().firstOrNull()?.refreshBalances()?.awaitAll()
    }

    private val assetsInfo: Flow<List<AssetInfo>> = currentWalletId()
        .flatMapLatest { walletId -> assetsDao.getAssetsInfo(walletId) }
        .toAssetInfoModel()
        .shareIn(scope, SharingStarted.Eagerly, replay = 1)

    fun getAssetsInfo(): Flow<List<AssetInfo>> = assetsInfo

    fun getAssetsInfo(walletId: WalletId): Flow<List<AssetInfo>> = assetsDao.getAssetsInfo(walletId.id)
        .toAssetInfoModel()
        .flowOn(Dispatchers.IO)

    fun getAssetsInfo(assetsId: List<AssetId>): Flow<List<AssetInfo>> = currentWalletId()
        .flatMapLatest { walletId -> assetsDao.getAssetsInfo(walletId, assetsId.map { it.toIdentifier() }) }
        .toAssetInfoModel()
        .flowOn(Dispatchers.IO)

    fun getAssetsInfoByChain(chain: Chain): Flow<List<AssetInfo>> = currentWalletId()
        .flatMapLatest { walletId -> assetsDao.getAssetsInfoByChain(walletId, chain) }
        .toAssetInfoModel()
        .flowOn(Dispatchers.IO)

    fun getHiddenAssetsInfoByChain(chain: Chain): Flow<List<AssetInfo>> = currentWalletId()
        .flatMapLatest { walletId -> assetsDao.getHiddenAssetsInfoByChain(walletId, chain) }
        .toAssetInfoModel()
        .flowOn(Dispatchers.IO)

    fun getAssetInfo(assetId: AssetId): Flow<AssetInfo?> {
        return currentWalletId()
            .flatMapLatest { walletId -> assetsDao.getAssetInfo(walletId, assetId.toIdentifier(), assetId.chain) }
            .map { it?.toDTO() }
            .flowOn(Dispatchers.IO)
    }

    fun asset(assetId: AssetId): Flow<Asset?> {
        return assetsDao.getAsset(assetId.toIdentifier())
            .map { it?.toDTO() }
            .flowOn(Dispatchers.IO)
    }

    fun getToken(assetId: AssetId): Flow<Asset?> = currentWalletId()
        .flatMapLatest { walletId -> assetsDao.getTokenInfo(walletId, assetId.toIdentifier(), assetId.chain) }
        .map { it?.toDTO()?.asset }
        .flowOn(Dispatchers.IO)

    fun getTokenInfo(assetId: AssetId): Flow<AssetInfo?> {
        return currentWalletId().flatMapLatest { walletId ->
            assetsDao.getAssetInfo(walletId, assetId.toIdentifier(), assetId.chain).flatMapLatest { assetInfo ->
                if (assetInfo == null) {
                    assetsDao.getTokenInfo(walletId, assetId.toIdentifier(), assetId.chain).map { it?.toDTO() }
                } else {
                    flow { emit(assetInfo.toDTO()) }
                }
            }
        }
        .flowOn(Dispatchers.IO)
    }

    fun getTokensInfo(assetsId: List<String>): Flow<List<AssetInfo>> {
        return currentWalletId()
            .flatMapLatest { walletId -> assetsDao.getAssetsInfoByAllWallets(walletId, assetsId) }
            .toAssetInfoModel()
    }

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

    suspend fun add(walletId: String, asset: Asset, visible: Boolean) {
        insertLocalAsset(walletId, asset, visible)
        if (visible) {
            streamSubscriptionService.addPrices(listOf(asset.id.toIdentifier()))
        }
    }

    suspend fun add(walletId: String, asset: AssetBasic, visible: Boolean) {
        insertAssetRecord(
            walletId = walletId,
            assetId = asset.asset.id,
            record = asset.toRecord(),
            visible = visible,
        )
        if (visible) {
            streamSubscriptionService.addPrices(listOf(asset.asset.id.toIdentifier()))
        }
    }

    suspend fun add(assets: List<AssetBasic>) = withContext(Dispatchers.IO) {
        if (assets.isEmpty()) {
            return@withContext
        }
        runCatching {
            assetsDao.insert(assets.map { it.toRecord() })
            assetsDao.updateBasicAssets(assets.map { it.toUpdateRecord() })
        }
            .onFailure { Log.e(TAG, "Failed to insert ${assets.size} assets", it) }
    }

    suspend fun linkAssetToWallet(
        walletId: String,
        assetId: AssetId,
        visible: Boolean,
    ) = withContext(Dispatchers.IO) {
        assetsDao.setWalletAssetVisibility(walletId, assetId.toIdentifier(), visible)
        if (visible) {
            streamSubscriptionService.addPrices(listOf(assetId.toIdentifier()))
        }
    }

    private suspend fun insertLocalAsset(walletId: String, asset: Asset, visible: Boolean) {
        assetsDao.insert(asset.defaultBasic.toRecord())
        assetsDao.setWalletAssetVisibility(walletId, asset.id.toIdentifier(), visible)
    }

    private suspend fun insertAssetRecord(
        walletId: String,
        assetId: AssetId,
        record: DbAsset,
        visible: Boolean,
    ) {
        val assetIdIdentifier = assetId.toIdentifier()
        // REPLACE would cascade-delete balances/accounts; insert-or-update keeps the asset row stable.
        assetsDao.insert(record)
        assetsDao.updateBasicAssets(listOf(record.toBasicUpdateRecord()))
        assetsDao.setWalletAssetVisibility(walletId, assetIdIdentifier, visible)
    }

    fun getAssetLinks(id: AssetId): Flow<List<AssetLink>> {
        return assetsDao.getAssetLinks(id.toIdentifier())
            .toAssetLinksModel()
            .flowOn(Dispatchers.IO)
    }

    fun getAssetMarket(id: AssetId): Flow<AssetMarket?> {
        return assetsDao.getAssetMarket(id.toIdentifier())
            .map { it?.toDTO() }
            .flowOn(Dispatchers.IO)
    }

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

private fun DbAsset.toBasicUpdateRecord() = DbAssetBasicUpdate(
    id = id,
    name = name,
    symbol = symbol,
    decimals = decimals,
    type = type,
    chain = chain,
    isEnabled = isEnabled,
    isBuyEnabled = isBuyEnabled,
    isSellEnabled = isSellEnabled,
    isSwapEnabled = isSwapEnabled,
    isStakeEnabled = isStakeEnabled,
    stakingApr = stakingApr,
    rank = rank,
)
