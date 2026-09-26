package com.gemwallet.android.ui.components.filters

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ui.LocalChainService
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ChainItem
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemChainRow

@Composable
fun rememberMatchingChains(availableChains: List<Chain>, query: String): List<GemChainRow> {
    val chainService = LocalChainService.current
    return remember(chainService, availableChains, query) {
        chainService.chainRows(availableChains.map { it.string }, query)
    }
}

fun LazyListScope.selectFilterChain(matchingChains: List<GemChainRow>, chainFilter: List<Chain>, onFilter: (Chain) -> Unit) {
    if (matchingChains.isEmpty()) {
        return
    }
    item {
        SubheaderItem(R.string.settings_networks_title)
    }
    val size = matchingChains.size
    matchingChains.forEachIndexed { index, row ->
        val chain = row.chain.requireChain()
        item {
            ChainItem(
                row = row,
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
