package com.gemwallet.android.features.wallet.viewmodels

import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.assets
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

internal class SearchContent<View>(val view: View, private val rows: List<AssetInfoDataAggregate>) {
    fun assets(ids: List<String>): List<AssetInfoDataAggregate> = rows.assets(ids) { it.asset.id }
}

internal fun <View, T> Flow<SearchContent<View>?>.select(scope: CoroutineScope, initial: T, value: (SearchContent<View>) -> T): StateFlow<T> = map { it?.let(value) ?: initial }
    .stateIn(scope, SharingStarted.Eagerly, initial)
