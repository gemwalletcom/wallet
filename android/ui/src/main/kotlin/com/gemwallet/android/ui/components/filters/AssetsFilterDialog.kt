package com.gemwallet.android.ui.components.filters

import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.list_item.SwitchProperty
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemAssetsFilterView

@Composable
fun AssetsFilter(isVisible: Boolean, availableChains: List<Chain>, filter: GemAssetsFilterView, onDismissRequest: () -> Unit, onChainFilter: (Chain) -> Unit, onBalanceFilter: (Boolean) -> Unit, onClearFilters: () -> Unit) {
    val query = rememberTextFieldState()

    val matchingChains = rememberMatchingChains(availableChains, query.text.toString())
    val chainFilter = remember(filter.selectedChains) { filter.selectedChains.map { it.requireChain() } }
    FormDialog(
        isVisible = isVisible,
        title = stringResource(R.string.filter_title),
        onDismiss = onDismissRequest,
        onClear = onClearFilters,
    ) {
        availableChains.takeIf { it.size > 1 }?.let { SearchBar(query) }
        if (filter.showsBalanceToggle) {
            HasBalances(isActive = filter.hasBalance, onBalanceFilter)
        }
        LazyColumn(modifier = Modifier.Companion.fillMaxSize()) {
            selectFilterChain(matchingChains, chainFilter, onChainFilter)
        }
    }
}

@Composable
private fun ColumnScope.HasBalances(isActive: Boolean, onBalanceFilter: (Boolean) -> Unit) {
    SwitchProperty(
        text = stringResource(R.string.filter_has_balance),
        checked = isActive,
        onCheckedChange = onBalanceFilter,
    )
}
