package com.gemwallet.android.features.onboarding.presents.import_wallet

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.AppUrl
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.features.onboarding.viewmodels.import_wallet.ImportWalletTypeViewModel
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.DocsInfoButton
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.list_item.ChainItem
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.DocsUrl
import uniffi.gemstone.GemImportWalletTypes

@Composable
fun ImportWalletTypeScreen(onClose: () -> Unit, onSelect: (ImportType) -> Unit) {
    val viewModel: ImportWalletTypeViewModel = hiltViewModel()
    val types by viewModel.types.collectAsStateWithLifecycle()

    ImportWalletTypeScene(
        types = types,
        chainFilter = viewModel.chainFilter,
        onSelect = onSelect,
        onClose = onClose,
    )
}

@Composable
private fun ImportWalletTypeScene(types: GemImportWalletTypes, chainFilter: TextFieldState, onSelect: (ImportType) -> Unit, onClose: () -> Unit) {
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
                    title = types.multicoin.string(LocalContext.current),
                    icon = R.drawable.multicoin_wallet,
                    listPosition = ListPosition.Single,
                ) {
                    onSelect(ImportType.phrase())
                }
            }
            itemsIndexed(types.chains) { index, row ->
                ChainItem(
                    row = row,
                    listPosition = ListPosition.getPosition(index, types.chains.size),
                ) {
                    onSelect(ImportType.phrase(row.chain.toChain()))
                }
            }
        }
    }
}
