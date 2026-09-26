package com.gemwallet.android.features.onboarding.presents.import_wallet

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
import com.gemwallet.android.AppUrl
import com.gemwallet.android.features.onboarding.presents.import_wallet.components.ImportInput
import com.gemwallet.android.features.onboarding.presents.import_wallet.components.ImportKindTab
import com.gemwallet.android.features.onboarding.viewmodels.import_wallet.ImportInputUIModel
import com.gemwallet.android.features.onboarding.viewmodels.import_wallet.ImportTabUIModel
import com.gemwallet.android.features.onboarding.viewmodels.import_wallet.ImportViewModel
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.ui.DetectScreenshot
import com.gemwallet.android.ui.DisableScreenShooting
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.fields.NameResolveIndicatorUIModel
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.sectionHeaderItem
import com.gemwallet.android.ui.components.parseMarkdownToAnnotatedString
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.gemwallet.android.ui.theme.space0
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemInfoTopic
import uniffi.gemstone.GemWalletImportKind

private val loadingDialogSize = 100.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ImportScreen(importType: ImportType, onImported: () -> Unit, onCancel: () -> Unit) {
    DisableScreenShooting()
    DetectScreenshot(AppUrl.howToSecureSecretPhrase)

    val viewModel: ImportViewModel = hiltViewModel()

    DisposableEffect(Unit) {
        viewModel.importSelect(importType)

        onDispose {}
    }
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val nameResolveIndicator by viewModel.nameResolveIndicator.collectAsStateWithLifecycle()
    val suggestions by viewModel.suggestions.collectAsStateWithLifecycle()
    val inputState = remember { mutableStateOf(TextFieldValue()) }

    ImportScene(
        inputState = inputState,
        importType = uiState.importType,
        tabs = uiState.tabs,
        input = uiState.input,
        title = uiState.title,
        showsTabs = uiState.showsTabs,
        nameResolveIndicator = nameResolveIndicator,
        dataError = uiState.dataError,
        buttonState = buttonState(loading = uiState.loading),
        onImport = { viewModel.import(onImported) },
        onInput = viewModel::onInput,
        onTypeChange = viewModel::importKind,
        suggestions = suggestions,
        onSelectSuggestion = viewModel::selectSuggestion,
        onCancel = onCancel,
    )
    if (uiState.loading) {
        Dialog(
            onDismissRequest = {},
            DialogProperties(dismissOnBackPress = false, dismissOnClickOutside = false),
        ) {
            Box(
                contentAlignment = Alignment.Center,
                modifier = Modifier
                    .size(loadingDialogSize)
                    .background(
                        MaterialTheme.colorScheme.background,
                        shape = RoundedCornerShape(paddingSmall),
                    ),
            ) {
                CircularProgressIndicator()
            }
        }
    }
    uiState.existingWalletName?.let { walletName ->
        InfoBottomSheet(
            item = GemInfoTopic.ExistingWalletImported(walletName).infoSheet {
                viewModel.dismissExistingWallet()
                onImported()
            },
            onClose = {
                viewModel.dismissExistingWallet()
                viewModel.clearInput()
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
    tabs: List<ImportTabUIModel>,
    input: ImportInputUIModel,
    title: String,
    showsTabs: Boolean,
    nameResolveIndicator: NameResolveIndicatorUIModel?,
    dataError: String?,
    buttonState: ButtonState,
    onImport: () -> Unit,
    onInput: (String, Int) -> Unit,
    onTypeChange: (ImportType) -> Unit,
    suggestions: List<String>,
    onSelectSuggestion: (String) -> String,
    onCancel: () -> Unit,
) {
    var dataErrorState by remember(dataError) { mutableStateOf(dataError) }

    Scene(
        title = title,
        onClose = onCancel,
        mainAction = {
            MainActionButton(
                title = stringResource(id = R.string.wallet_import_action),
                state = buttonState,
                onClick = onImport,
            )
        },
    ) {
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
        ) {
            item {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .listItem(ListPosition.Single)
                        .padding(sceneContentPadding())
                        .padding(bottom = space0),
                    verticalArrangement = Arrangement.spacedBy(paddingHalfSmall),
                ) {
                    TypeSelection(tabs, showsTabs) { type ->
                        onTypeChange(type)
                        inputState.value = TextFieldValue()
                    }
                    DataInput(input, inputState, nameResolveIndicator, suggestions, onSelectSuggestion, onInput) {
                        dataErrorState = null
                    }
                    ErrorMessage(dataErrorState)
                }
            }
            if (input.showsViewOnlyWarning) {
                item {
                    Text(
                        modifier = Modifier.sectionHeaderItem(),
                        text = parseMarkdownToAnnotatedString(
                            stringResource(R.string.wallet_import_address_warning),
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
    input: ImportInputUIModel,
    inputState: MutableState<TextFieldValue>,
    nameResolveIndicator: NameResolveIndicatorUIModel?,
    suggestions: List<String>,
    onSelectSuggestion: (String) -> String,
    onInput: (String, Int) -> Unit,
    onChange: () -> Unit,
) {
    ImportInput(
        inputState = inputState.value,
        input = input,
        indicator = nameResolveIndicator,
        onValueChange = { query ->
            inputState.value = query
            onChange()
            onInput(query.text, query.selection.start)
        },
    )

    if (suggestions.isNotEmpty() && input.supportsPhraseSuggestions) {
        LazyRow(
            horizontalArrangement = Arrangement.spacedBy(paddingSmall),
        ) {
            items(suggestions) { word ->
                SuggestionChip(
                    onClick = {
                        val text = onSelectSuggestion(word)
                        inputState.value = TextFieldValue(text = text, selection = TextRange(text.length))
                        onChange()
                    },
                    label = { Text(text = word) },
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun TypeSelection(tabs: List<ImportTabUIModel>, showsTabs: Boolean, onTypeChange: (ImportType) -> Unit) {
    if (!showsTabs) {
        return
    }
    PrimaryTabRow(
        modifier = Modifier.fillMaxWidth().clip(RoundedCornerShape(paddingHalfSmall)),
        selectedTabIndex = tabs.indexOfFirst { it.isSelected }.coerceAtLeast(0),
        indicator = { Box {} },
        containerColor = Color.Transparent,
        divider = {},
    ) {
        tabs.forEach { tab ->
            ImportKindTab(tab, onTypeChange)
        }
    }
    Spacer16()
}

@Composable
private fun ErrorMessage(error: String?) {
    error ?: return
    Text(text = error, color = MaterialTheme.colorScheme.error)
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
                importType = ImportType(GemWalletImportKind.ADDRESS, Chain.Bitcoin),
                tabs = listOf(
                    ImportTabUIModel(ImportType(GemWalletImportKind.PHRASE, Chain.Bitcoin), R.string.common_phrase, isSelected = false),
                    ImportTabUIModel(ImportType(GemWalletImportKind.ADDRESS, Chain.Bitcoin), R.string.common_address, isSelected = true),
                ),
                input = ImportInputUIModel(
                    placeholder = R.string.wallet_import_address_field,
                    protectsInput = false,
                    supportsPhraseSuggestions = false,
                    showsViewOnlyWarning = true,
                ),
                title = "Ethereum",
                showsTabs = true,
                nameResolveIndicator = null,
                dataError = null,
                buttonState = ButtonState.Enabled,
                onImport = {},
                onInput = { _, _ -> },
                onTypeChange = {},
                suggestions = emptyList(),
                onSelectSuggestion = { "" },
                onCancel = {},
            )
        }
    }
}
