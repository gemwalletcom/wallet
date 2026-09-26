package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregates
import com.wallet.core.primitives.AssetData
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.stateIn
import java.util.Locale

class GetActiveAssetsInfoImpl(getWalletAssets: GetWalletAssets, getCurrentCurrency: GetCurrentCurrency, userConfig: UserConfig, scope: CoroutineScope = CoroutineScope(Dispatchers.Default)) : GetActiveAssetsInfo {

    private val rows = AssetRows()

    private val assetsInfo: StateFlow<List<AssetInfoDataAggregate>> =
        combine(getWalletAssets(), getCurrentCurrency.getCurrency(), userConfig.isHideBalances()) { items, currency, hideBalance ->
            rows.aggregates(items, currency, hideBalance)
        }
            .distinctUntilChanged()
            .stateIn(scope, SharingStarted.Eagerly, emptyList())

    override fun assetsInfo(): StateFlow<List<AssetInfoDataAggregate>> = assetsInfo
}

internal class AssetRows {

    private data class Presentation(val currency: Currency, val hideBalance: Boolean, val locale: Locale)

    private var presentation: Presentation? = null
    private var previous: Map<AssetData, AssetInfoDataAggregate> = emptyMap()

    fun aggregates(items: List<AssetData>, currency: Currency, hideBalance: Boolean): List<AssetInfoDataAggregate> {
        val presentation = Presentation(currency = currency, hideBalance = hideBalance, locale = Locale.getDefault())
        val reused = if (presentation == this.presentation) previous else emptyMap()
        val missing = items.filterNot(reused::containsKey).distinct()
        val built = missing.zip(missing.toAssetInfoDataAggregates(currency, hideBalance = hideBalance)).toMap()
        val aggregates = items.map { reused[it] ?: built.getValue(it) }
        this.presentation = presentation
        previous = items.zip(aggregates).toMap()
        return aggregates
    }
}
