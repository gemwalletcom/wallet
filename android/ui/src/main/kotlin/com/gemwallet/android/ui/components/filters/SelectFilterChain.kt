package com.gemwallet.android.ui.components.filters

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.ui.Modifier
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.assetType
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ChainItem
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.Chain
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import com.gemwallet.android.ui.LocalChainService

@Composable
fun rememberMatchingChains(availableChains: List<Chain>, query: String): List<Chain> {
    val chainService = LocalChainService.current
    return remember(chainService, availableChains, query) {
        val matching = chainService.getMatchingChains(availableChains.map { it.string }, query).map { it.requireChain() }
        availableChains.filter { chain ->
            matching.contains(chain) || chain.assetType()?.string?.contains(query, ignoreCase = true) == true
        }
    }
}

fun LazyListScope.selectFilterChain(
    matchingChains: List<Chain>,
    chainFilter: List<Chain>,
    onFilter: (Chain) -> Unit,
) {
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
                }
            ) { onFilter(chain) }
        }
    }
}
