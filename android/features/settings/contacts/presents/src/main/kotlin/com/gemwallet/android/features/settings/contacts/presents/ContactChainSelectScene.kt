package com.gemwallet.android.features.settings.contacts.presents

import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.gemwallet.android.features.settings.contacts.viewmodels.ContactChainSelectViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.SelectChain
import com.wallet.core.primitives.Chain

@Composable
fun ContactChainSelectScene(onSelect: (Chain) -> Unit, onCancel: () -> Unit, viewModel: ContactChainSelectViewModel = hiltViewModel()) {
    val chainFilter = rememberTextFieldState()
    val query = chainFilter.text.toString()

    SelectChain(
        chains = remember(query) { viewModel.chains(query) },
        chainFilter = chainFilter,
        title = stringResource(R.string.transfer_network),
        onSelect = onSelect,
        onCancel = onCancel,
    )
}
