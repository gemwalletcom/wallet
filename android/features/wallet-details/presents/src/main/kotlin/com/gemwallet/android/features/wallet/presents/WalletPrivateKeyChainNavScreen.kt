package com.gemwallet.android.features.wallet.presents

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.SelectChain
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemChainService

@Composable
fun WalletPrivateKeyChainNavScreen(
    chains: List<Chain>,
    onSelect: (Chain) -> Unit,
    onCancel: () -> Unit,
) {
    BackHandler { onCancel() }

    val chainFilter = rememberTextFieldState()
    val query = chainFilter.text.toString()
    val chainService = remember { GemChainService() }
    val matchingChains = remember(chains, query) {
        chainService.getMatchingChains(chains.map { it.string }, query).map { it.requireChain() }
    }

    SelectChain(
        chains = matchingChains,
        chainFilter = chainFilter,
        title = stringResource(R.string.common_private_key),
        onSelect = onSelect,
        onCancel = onCancel,
    )
}
