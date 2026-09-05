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
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import uniffi.gemstone.GemWalletImportException
import com.gemwallet.android.application.wallet_import.values.WalletImportResult
import com.gemwallet.android.features.import_wallet.components.ImportInput
import com.gemwallet.android.features.import_wallet.components.WalletTypeTab
import com.gemwallet.android.features.import_wallet.components.importTypeTabIndex
import com.gemwallet.android.features.import_wallet.components.importWalletTabs
import com.gemwallet.android.features.import_wallet.components.supportsPhraseSuggestions
import com.gemwallet.android.features.import_wallet.viewmodels.ImportViewModel
import com.gemwallet.android.AppUrl
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.ui.DetectScreenshot
import com.gemwallet.android.ui.DisableScreenShooting
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.sectionHeaderItem
import com.gemwallet.android.ui.components.parseMarkdownToAnnotatedString
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.wallet.core.primitives.Chain
import com.gemwallet.android.ui.models.name.NameRecordState
import com.wallet.core.primitives.WalletType
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import uniffi.gemstone.DocsUrl

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
    onImported: (WalletImportResult) -> Unit,
    onCancel: () -> Unit
) {
    DisableScreenShooting()
    DetectScreenshot(AppUrl.docs(DocsUrl.HowToSecureSecretPhrase))

    val viewModel: ImportViewModel = hiltViewModel()

    DisposableEffect(Unit) {
        viewModel.importSelect(importType)

        onDispose {}
    }
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val nameResolveState by viewModel.nameResolveState.collectAsStateWithLifecycle()
    val inputState = remember { mutableStateOf(TextFieldValue()) }

    ImportScene(
        inputState = inputState,
        importType = uiState.importType,
        defaultWalletName = uiState.defaultWalletName,
        chainName = uiState.chainName,
        nameResolveState = nameResolveState,
        dataError = uiState.dataError,
        buttonState = buttonState(loading = uiState.loading),
        onImport = { generatedName, value ->
            viewModel.import(generatedName, value, onImported)
        },
        onInput = viewModel::onInput,
        onTypeChange = viewModel::chainType,
        invalidWords = viewModel::invalidPhraseWords,
        phraseSuggestions = viewModel::phraseSuggestions,
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
    uiState.existingWalletResult?.let { result ->
        InfoBottomSheet(
            item = InfoSheetEntity.ExistingWalletImported(
                walletName = result.wallet.name,
                actionLabel = stringResource(R.string.common_continue),
                action = {
                    viewModel.dismissExistingWallet()
                    onImported(result)
                },
            ),
            onClose = {
                viewModel.dismissExistingWallet()
                inputState.value = TextFieldValue()
            },
        )
    }

}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ImportScene(
    inputState: MutableState<TextFieldValue>,
    importType: ImportType,
    defaultWalletName: String,
    chainName: String,
    nameResolveState: NameRecordState,
    dataError: Throwable?,
    buttonState: ButtonState,
    onImport: (generatedName: String, value: String) -> Unit,
    onInput: (String) -> Unit,
    onTypeChange: (WalletType) -> Unit,
    invalidWords: (String) -> Set<String>,
    phraseSuggestions: (String) -> List<String>,
    onCancel: () -> Unit
) {
    val title = when (val sceneTitle = importSceneTitle(importType, chainName)) {
        is ImportSceneTitle.Resource -> stringResource(sceneTitle.resId)
        is ImportSceneTitle.Text -> sceneTitle.value
    }
    val generatedName = defaultWalletName
    var dataErrorState by remember(dataError) { mutableStateOf(dataError) }

    Scene(
        title = title,
        onClose = onCancel,
        mainAction = {
            MainActionButton(
                title = stringResource(id = R.string.wallet_import_action),
                state = buttonState,
                onClick = {
                    onImport(generatedName, inputState.value.text)
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
                    DataInput(importType, inputState, nameResolveState, invalidWords, phraseSuggestions, onInput) {
                        dataErrorState = null
                    }
                    ErrorMessage(dataErrorState)
                }
            }
            if (importType.walletType == WalletType.View) {
                item {
                    Text(
                        modifier = Modifier.sectionHeaderItem(),
                        text = parseMarkdownToAnnotatedString(
                            stringResource(R.string.wallet_import_address_warning)
                        ),
                        color = MaterialTheme.colorScheme.secondary,
                        style = MaterialTheme.typography.bodySmall,
                    )
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
    nameResolveState: NameRecordState,
    invalidWords: (String) -> Set<String>,
    phraseSuggestions: (String) -> List<String>,
    onInput: (String) -> Unit,
    onChange: () -> Unit,
) {
    val suggestions = remember(importType.walletType) { mutableStateListOf<String>() }

    ImportInput(
        invalidWords = invalidWords,
        inputState = inputState.value,
        importType = importType,
        uiState = nameResolveState,
        onValueChange = { query ->
            inputState.value = query
            suggestions.clear()

            onChange()
            onInput(query.text)

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
            val result = phraseSuggestions(word)
            suggestions.addAll(result)
        },
    )

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
        selectedTabIndex = importTypeTabIndex(importType.walletType, importType.chain),
        indicator = { Box {} },
        containerColor = Color.Transparent,
        divider = {}
    ) {
        importWalletTabs(importType.chain).forEach { walletType ->
            WalletTypeTab(walletType, importType.walletType, onTypeChange)
        }
    }
    Spacer16()
}

@Composable
private fun ErrorMessage(error: Throwable?) {
    val text = when (error) {
        is GemWalletImportException.InvalidSecretPhraseWords -> stringResource(
            R.string.errors_import_invalid_secret_phrase_word,
            error.words.joinToString()
        )
        is GemWalletImportException.InvalidSecretPhrase -> stringResource(R.string.errors_import_invalid_secret_phrase)
        is GemWalletImportException.InvalidAddress -> stringResource(R.string.errors_invalid_address_name)
        is GemWalletImportException.InvalidPrivateKey -> stringResource(R.string.errors_import_invalid_private_key)
        null -> return
        else -> stringResource(
            R.string.errors_create_wallet,
            error.message?.takeIf { it.isNotBlank() } ?: stringResource(R.string.errors_unknown_try_again),
        )
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
                inputState = remember { mutableStateOf(TextFieldValue()) },
                importType = ImportType(chain = Chain.Bitcoin, walletType = WalletType.View),
                defaultWalletName = "Wallet #1",
                chainName = "Ethereum",
                nameResolveState = NameRecordState.None,
                dataError = null,
                buttonState = ButtonState.Enabled,
                onImport = {_, _ -> },
                onInput = {},
                onTypeChange = {},
                invalidWords = { emptySet() },
                phraseSuggestions = { emptyList() },
                onCancel = {},
            )
        }
    }
}
