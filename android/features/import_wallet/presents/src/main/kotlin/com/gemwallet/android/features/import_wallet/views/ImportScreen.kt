package com.gemwallet.android.features.import_wallet.views

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.PrimaryTabRow
import androidx.compose.material3.SuggestionChip
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.blockchain.operators.walletcore.WCFindPhraseWord
import com.gemwallet.android.cases.wallet.ImportError
import com.gemwallet.android.features.import_wallet.components.ImportInput
import com.gemwallet.android.features.import_wallet.components.WalletTypeTab
import com.gemwallet.android.features.import_wallet.components.importTypeTabIndex
import com.gemwallet.android.features.import_wallet.components.importWalletTabs
import com.gemwallet.android.features.import_wallet.components.supportsPhraseSuggestions
import com.gemwallet.android.features.import_wallet.viewmodels.ImportViewModel
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.ui.DisableScreenShooting
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.parseMarkdownToAnnotatedString
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord
import com.wallet.core.primitives.WalletType

internal sealed interface ImportSceneTitle {
    data class Resource(val resId: Int) : ImportSceneTitle
    data class Text(val value: String) : ImportSceneTitle
}

internal fun importSceneTitle(importType: ImportType, chainName: String): ImportSceneTitle {
    return when (importType.walletType) {
        WalletType.Multicoin -> ImportSceneTitle.Resource(R.string.wallet_multicoin)
        WalletType.Single,
        WalletType.PrivateKey,
        WalletType.View -> ImportSceneTitle.Text(chainName)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ImportScreen(
    importType: ImportType,
    onImported: (walletId: String) -> Unit,
    onCancel: () -> Unit
) {
    DisableScreenShooting()

    val viewModel: ImportViewModel = hiltViewModel()

    DisposableEffect(Unit) {
        viewModel.importSelect(importType)

        onDispose {}
    }
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    ImportScene(
        importType = uiState.importType,
        generatedNameIndex = uiState.generatedNameIndex,
        chainName = uiState.chainName,
        nameRecord = uiState.nameRecord,
        dataError = uiState.dataError,
        onImport = { generatedName, value, nameRecord ->
            viewModel.import(generatedName, value, nameRecord, onImported)
        },
        onTypeChange = viewModel::chainType,
        onCancel = onCancel,
    )
    if (uiState.loading) {
        Dialog(
            onDismissRequest = {},
            DialogProperties(dismissOnBackPress = false, dismissOnClickOutside = false)
        ) {
            Box(
                contentAlignment = Alignment.Center,
                modifier = Modifier
                    .size(100.dp)
                    .background(
                        MaterialTheme.colorScheme.background,
                        shape = RoundedCornerShape(8.dp)
                    )
            ) {
                CircularProgressIndicator()
            }
        }
    }

}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ImportScene(
    importType: ImportType,
    generatedNameIndex: Int,
    chainName: String,
    nameRecord: NameRecord?,
    dataError: ImportError?,
    onImport: (generatedName: String, value: String, nameRecord: NameRecord?) -> Unit,
    onTypeChange: (WalletType) -> Unit,
    onCancel: () -> Unit
) {
    val inputState = remember {
        mutableStateOf(TextFieldValue())
    }
    val title = when (val sceneTitle = importSceneTitle(importType, chainName)) {
        is ImportSceneTitle.Resource -> stringResource(sceneTitle.resId)
        is ImportSceneTitle.Text -> sceneTitle.value
    }
    val chainWalletName = stringResource(id = R.string.wallet_default_name_chain, chainName, generatedNameIndex)
    val mainWalletName = stringResource(id = R.string.wallet_default_name, generatedNameIndex)
    val generatedName = remember(key1 = importType.walletType, key2 = generatedNameIndex) {
        if (generatedNameIndex == 0 || importType.walletType == WalletType.Multicoin) {
            mainWalletName
        } else {
            chainWalletName
        }
    }
    val nameRecordState = remember(nameRecord?.address) { mutableStateOf(nameRecord) }
    var dataErrorState by remember(dataError) { mutableStateOf(dataError) }

    Scene(
        title = title,
        onClose = onCancel,
        mainAction = {
            MainActionButton(
                title = stringResource(id = R.string.wallet_import_action),
                onClick = {
                    onImport(generatedName, inputState.value.text, nameRecordState.value)
                },
            )
        },
    ) {
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
        ) {
            item {
                Column (
                    modifier = Modifier
                        .fillMaxWidth()
                        .listItem(ListPosition.Single)
                        .padding(sceneContentPadding())
                        .padding(bottom = 0.dp),
                    verticalArrangement = Arrangement.spacedBy(4.dp)
                ) {
                    TypeSelection(importType) { walletType ->
                        onTypeChange(walletType)
                        inputState.value = TextFieldValue()
                    }
                    DataInput(importType, inputState, nameRecordState) {
                        dataErrorState = null
                    }
                    ErrorMessage(dataErrorState)
                }
            }
            item { Spacer(modifier = Modifier.size(it.calculateBottomPadding())) }
        }
    }
}

@Composable
private fun DataInput(
    importType: ImportType,
    inputState: MutableState<TextFieldValue>,
    nameRecordState: MutableState<NameRecord?>,
    onChange: () -> Unit,
) {
    val suggestions = remember(importType.walletType) { mutableStateListOf<String>() }

    ImportInput(
        inputState = inputState.value,
        importType = importType,
        onValueChange = { query ->
            inputState.value = query
            suggestions.clear()

            onChange()

            if (!supportsPhraseSuggestions(importType.walletType)) {
                return@ImportInput
            }

            val cursorPosition = query.selection.start
            if (query.text.isEmpty()) {
                return@ImportInput
            }
            val word = query.text.substring(0..<cursorPosition).split(" ")
                .lastOrNull()
            if (word.isNullOrEmpty()) {
                return@ImportInput
            }
            val result = WCFindPhraseWord().invoke(word)
            suggestions.addAll(result)
        }
    ) {
        nameRecordState.value = it
    }

    if (importType.walletType == WalletType.View) {
        Text(
            modifier = Modifier.fillMaxWidth(),
            text = parseMarkdownToAnnotatedString(
                stringResource(R.string.wallet_import_address_warning)
            ),
            color = MaterialTheme.colorScheme.secondary,
            style = MaterialTheme.typography.bodySmall,
        )
    }
    if (suggestions.isNotEmpty() && supportsPhraseSuggestions(importType.walletType)) {
        LazyRow(
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            items(suggestions) { word ->
                SuggestionChip(
                    onClick = {
                        val processed = setSuggestion(inputState.value, word)
                        inputState.value = processed
                        suggestions.clear()
                        onChange()
                    },
                    label = { Text(text = word) }
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun TypeSelection(
    importType: ImportType,
    onTypeChange: (WalletType) -> Unit,
) {
    if (importType.walletType == WalletType.Multicoin) {
        return
    }
    PrimaryTabRow(
        modifier = Modifier.fillMaxWidth().clip(RoundedCornerShape(4.dp)),
        selectedTabIndex = importTypeTabIndex(importType.walletType),
        indicator = { Box {} },
        containerColor = Color.Transparent,//(0xFFEBEBEB),
        divider = {}
    ) {
        importWalletTabs.forEach { walletType ->
            WalletTypeTab(walletType, importType.walletType, onTypeChange)
        }
    }
    Spacer16()
}

@Composable
private fun ErrorMessage(error: ImportError?) {
    val text = when (error) {
        is ImportError.CreateError -> stringResource(R.string.errors_create_wallet, error.message ?: "")
        is ImportError.InvalidWords -> stringResource(
            R.string.errors_import_invalid_secret_phrase_word,
            error.words.joinToString()
        )
        ImportError.InvalidationSecretPhrase -> stringResource(R.string.errors_import_invalid_secret_phrase)
        ImportError.InvalidAddress -> stringResource(R.string.errors_invalid_address_name)
        ImportError.InvalidationPrivateKey -> "Invalid private key"
        is ImportError.DuplicatedWallet -> "Duplicated wallet"
        null -> return
    }
    Text(text = text, color = MaterialTheme.colorScheme.error)
}

private fun setSuggestion(inputState: TextFieldValue, word: String): TextFieldValue {
    val cursorPosition = inputState.selection.start
    val inputFull = inputState.text
    val rightInput =
        inputState.text.substring(0..<cursorPosition)
    val leftInput = inputState.text.substring(cursorPosition)
    val lastInput = rightInput.split(" ").lastOrNull() ?: ""
    val phrase = rightInput.removeSuffix(lastInput)
    return TextFieldValue(
        text = inputFull.replaceRange(0, inputFull.length, "$phrase$word $leftInput"),
        selection = TextRange("$phrase$word ".length)
    )
}

@Composable
@Preview(device = Devices.NEXUS_6)
@Preview(device = Devices.NEXUS_7)
@Preview(showBackground = true, device = Devices.NEXUS_7)
@Preview(showBackground = true, device = Devices.NEXUS_5)
@Preview(showBackground = true, device = "spec:width=411dp,height=891dp")
fun PreviewImportAddress() {
    WalletTheme {
        Box(modifier = Modifier.fillMaxSize()) {
            ImportScene(
                importType = ImportType(chain = Chain.Bitcoin, walletType = WalletType.View),
                generatedNameIndex = 1,
                chainName = "Ethereum",
                nameRecord = null,
                dataError = null,
                onImport = {_, _, _ -> },
                onTypeChange = {},
                onCancel = {},
            )
        }
    }
}
