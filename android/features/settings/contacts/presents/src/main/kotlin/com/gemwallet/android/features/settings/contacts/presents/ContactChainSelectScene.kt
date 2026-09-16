package com.gemwallet.android.features.settings.contacts.presents

import androidx.activity.compose.BackHandler
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.contacts.viewmodels.ContactChainSelectViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.SelectChain
import com.wallet.core.primitives.Chain

@Composable
fun ContactChainSelectScene(
    onSelect: (Chain) -> Unit,
    onCancel: () -> Unit,
    viewModel: ContactChainSelectViewModel = hiltViewModel(),
) {
    BackHandler { onCancel() }

    val chains by viewModel.chains.collectAsStateWithLifecycle()

    SelectChain(
        chains = chains,
        chainFilter = viewModel.chainFilter,
        title = stringResource(R.string.transfer_network),
        onSelect = onSelect,
        onCancel = onCancel,
    )
}
