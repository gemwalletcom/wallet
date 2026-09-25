package com.gemwallet.android.ui.components.filters

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ui.LocalChainService
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ChainItem
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.Chain

@Composable
fun rememberMatchingChains(availableChains: List<Chain>, query: String): List<Chain> {
    val chainService = LocalChainService.current
    return remember(chainService, availableChains, query) {
        chainService.getMatchingChains(availableChains.map { it.string }, query).map { it.requireChain() }
    }
}

fun LazyListScope.selectFilterChain(matchingChains: List<Chain>, chainFilter: List<Chain>, onFilter: (Chain) -> Unit) {
    val items = matchingChains.map { it.asset() }
    if (items.isEmpty()) {
        return
    }
    item {
        SubheaderItem(R.string.settings_networks_title)
    }
    val size = items.size
    items.forEachIndexed { index, item ->
        val chain = item.id.chain
        item {
            ChainItem(
                title = chain.networkName(),
                icon = chain,
                listPosition = ListPosition.getPosition(index, size),
                trailing = {
                    if (chainFilter.contains(chain)) {
                        SelectionCheckmark(modifier = Modifier.padding(end = paddingSmall))
                    }
                },
            ) { onFilter(chain) }
        }
    }
}
