package com.gemwallet.android.features.settings.contacts.presents

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.filter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.SelectChain
import com.wallet.core.primitives.Chain

@Composable
fun ContactChainSelectScene(
    onSelect: (Chain) -> Unit,
    onCancel: () -> Unit,
) {
    BackHandler { onCancel() }

    val chainFilter = rememberTextFieldState()
    val query = chainFilter.text.toString()
    val chains = remember(query) { Chain.entries.toList().filter(query.lowercase()) }

    SelectChain(
        chains = chains,
        chainFilter = chainFilter,
        title = stringResource(R.string.transfer_network),
        onSelect = onSelect,
        onCancel = onCancel,
    )
}
