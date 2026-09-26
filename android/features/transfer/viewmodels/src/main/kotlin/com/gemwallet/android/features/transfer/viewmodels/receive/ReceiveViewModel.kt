package com.gemwallet.android.features.transfer.viewmodels.receive

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.AssetQueryOptional
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import dagger.assisted.Assisted
import dagger.assisted.AssistedFactory
import dagger.assisted.AssistedInject
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemCopy
import uniffi.gemstone.GemReceiveNetwork
import uniffi.gemstone.GemReceiveNetworks
import uniffi.gemstone.GemReceiveServiceInterface
import uniffi.gemstone.GemReceiveWarning
import uniffi.gemstone.addressCopy

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel(assistedFactory = ReceiveViewModel.Factory::class)
class ReceiveViewModel @AssistedInject constructor(
    @Assisted private val sourceAssetId: AssetId,
    private val assetQuery: AssetQueryOptional,
    private val getWalletAssets: GetWalletAssets,
    private val service: GemReceiveServiceInterface,
    getSession: GetSession,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @ApplicationContext private val context: Context,
) : ViewModel() {

    private val selectedAssetId = MutableStateFlow(sourceAssetId)
    private val session = getSession()

    val asset = selectedAssetId
        .flatMapLatest { assetId ->
            session.filterNotNull().flatMapLatest { session ->
                assetQuery(session.wallet.id.id, assetId).map { info ->
                    val account = info?.account?.takeIf { it.address.isEmpty() }?.let { session.wallet.getAccount(info.asset.chain) }
                    if (info == null || account == null) info else info.copy(account = account)
                }
            }
        }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, storedAsset(sourceAssetId))

    private fun storedAsset(assetId: AssetId) = getWalletAssets().value.firstOrNull { it.asset.id == assetId }

    val networks = combine(
        asset.filterNotNull().filter { it.asset.id == sourceAssetId },
        session.filterNotNull(),
    ) { assetInfo, session ->
        service.networks(
            assetInfo.asset.toGem(),
            assetInfo.associations.map { it.assetId.toIdentifier() },
            session.wallet.toGem(),
        )
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, GemReceiveNetworks(networks = listOf(GemReceiveNetwork(sourceAssetId.toIdentifier(), standard = null)), showsSelector = false))

    fun warnings(chain: Chain): List<GemReceiveWarning> = service.warnings(chain.string)

    fun warningText(asset: Asset): String = warnings(asset.id.chain).joinToString(" ") { it.text(context, asset) }

    fun selectAsset(assetId: AssetId) {
        selectedAssetId.value = assetId
    }

    fun shareAddress(): String? = asset.value?.account?.address?.takeIf { it.isNotEmpty() }

    fun copyAddress(): GemCopy? = asset.value?.let { assetInfo ->
        assetInfo.account.address.takeIf { it.isNotEmpty() }?.let { addressCopy(assetInfo.asset.id.chain.string, it) }
    }

    @AssistedFactory
    interface Factory {
        fun create(assetId: AssetId): ReceiveViewModel
    }

    fun setVisible() = viewModelScope.launch(ioDispatcher) {
        val assetId = asset.value?.asset?.id ?: return@launch
        val wallet = session.filterNotNull().first().wallet
        runCatchingCancellable { service.enableAsset(wallet.id.id, assetId.toIdentifier()) }
            .onFailure { Log.e(TAG, "enabling ${assetId.toIdentifier()} failed", it) }
    }
}

private const val TAG = "Receive"
