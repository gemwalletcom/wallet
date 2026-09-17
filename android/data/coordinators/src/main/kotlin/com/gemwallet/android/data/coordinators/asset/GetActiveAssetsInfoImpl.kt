package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregate
import com.gemwallet.android.domains.price.values.RowFormatters
import com.gemwallet.android.model.AssetInfo
import uniffi.gemstone.GemAssetRowStyle
import uniffi.gemstone.GemAssetTitleStyle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.stateIn
import java.util.Locale

class GetActiveAssetsInfoImpl(
    getWalletAssets: GetWalletAssets,
    userConfig: UserConfig,
    rowStyle: GemAssetRowStyle,
    scope: CoroutineScope = CoroutineScope(Dispatchers.Default),
) : GetActiveAssetsInfo {

    private val rows = AssetRows(rowStyle.title)

    private val assetsInfo: StateFlow<List<AssetInfoDataAggregate>> =
        combine(getWalletAssets(), userConfig.isHideBalances()) { items, hideBalance ->
            rows.aggregates(items, hideBalance)
        }
        .distinctUntilChanged()
        .stateIn(scope, SharingStarted.Eagerly, emptyList())

    override fun assetsInfo(): StateFlow<List<AssetInfoDataAggregate>> = assetsInfo
}

internal class AssetRows(private val naming: GemAssetTitleStyle) {

    private data class Presentation(val hideBalance: Boolean, val locale: Locale)

    private var presentation: Presentation? = null
    private var previous: Map<AssetInfo, AssetInfoDataAggregate> = emptyMap()

    fun aggregates(items: List<AssetInfo>, hideBalance: Boolean): List<AssetInfoDataAggregate> {
        val presentation = Presentation(hideBalance = hideBalance, locale = Locale.getDefault())
        val reused = if (presentation == this.presentation) previous else emptyMap()
        val missing = items.filterNot(reused::containsKey).distinct()
        val built = if (missing.isEmpty()) {
            emptyMap()
        } else {
            val formatters = RowFormatters()
            missing.associateWith { it.toAssetInfoDataAggregate(naming = naming, hideBalance = hideBalance, formatters = formatters) }
        }
        val aggregates = items.map { reused[it] ?: built.getValue(it) }
        this.presentation = presentation
        previous = items.zip(aggregates).toMap()
        return aggregates
    }
}
