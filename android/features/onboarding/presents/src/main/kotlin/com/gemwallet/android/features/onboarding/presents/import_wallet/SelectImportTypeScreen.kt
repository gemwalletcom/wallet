package com.gemwallet.android.features.onboarding.presents.import_wallet

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.AppUrl
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.features.onboarding.viewmodels.import_wallet.SelectImportTypeViewModel
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.DocsInfoButton
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.list_item.ChainItem
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.Chain
import uniffi.gemstone.DocsUrl

@Composable
fun SelectImportTypeScreen(onClose: () -> Unit, onSelect: (ImportType) -> Unit) {
    val viewModel: SelectImportTypeViewModel = hiltViewModel()
    val chains by viewModel.chains.collectAsStateWithLifecycle()

    SelectImportTypeScene(
        chains = chains,
        chainFilter = viewModel.chainFilter,
        onSelect = onSelect,
        onClose = onClose,
    )
}

@Composable
private fun SelectImportTypeScene(chains: List<Chain>, chainFilter: TextFieldState, onSelect: (ImportType) -> Unit, onClose: () -> Unit) {
    Scene(
        title = stringResource(id = R.string.wallet_import_title),
        actions = {
            DocsInfoButton(AppUrl.docs(DocsUrl.MigrateWallet))
        },
        onClose = onClose,
    ) {
        LazyColumn(modifier = Modifier) {
            item {
                SearchBar(query = chainFilter)
            }
            item {
                ChainItem(
                    modifier = Modifier.testTag("multicoin_item"),
                    title = stringResource(id = R.string.wallet_multicoin),
                    icon = R.drawable.multicoin_wallet,
                    listPosition = ListPosition.Single,
                ) {
                    onSelect(ImportType.phrase())
                }
            }
            itemsIndexed(chains) { index, chain ->
                ChainItem(
                    title = chain.networkName(),
                    icon = chain,
                    listPosition = ListPosition.getPosition(index, chains.size),
                ) {
                    onSelect(ImportType.phrase(chain))
                }
            }
        }
    }
}

@Preview
@Composable
fun PreviewChainSelectScreen() {
    MaterialTheme {
        Column {
            SelectImportTypeScene(
                chains = listOf(Chain.Bitcoin, Chain.Ethereum, Chain.Solana),
                chainFilter = rememberTextFieldState(),
                onClose = {},
                onSelect = {},
            )
        }
    }
}
