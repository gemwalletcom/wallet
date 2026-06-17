package com.gemwallet.android.features.import_wallet.components

import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.PlatformImeOptions
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.blockchain.operators.InvalidWords
import com.gemwallet.android.blockchain.operators.gemstone.GemValidatePhraseOperator
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.FieldBottomAction
import com.gemwallet.android.ui.components.clipboard.clear
import com.gemwallet.android.ui.components.clipboard.getPlainText
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.features.recipient.viewmodel.AddressChainViewModel
import com.gemwallet.android.features.recipient.viewmodel.NameRecordState
import com.wallet.core.primitives.NameRecord
import com.wallet.core.primitives.WalletType

internal fun supportsPhraseSuggestions(walletType: WalletType): Boolean {
    return when (walletType) {
        WalletType.Multicoin,
        WalletType.Single -> true
        WalletType.PrivateKey,
        WalletType.View -> false
    }
}

internal fun shouldProtectInput(walletType: WalletType): Boolean {
    return when (walletType) {
        WalletType.Multicoin,
        WalletType.Single,
        WalletType.PrivateKey -> true
        WalletType.View -> false
    }
}

@Composable
internal fun ImportInput(
    inputState: TextFieldValue,
    importType: ImportType,
    onValueChange: (TextFieldValue) -> Unit,
    onResolved: (NameRecord?) -> Unit,
) {
    val viewModel: AddressChainViewModel = hiltViewModel()
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    fun updateInput(value: TextFieldValue) {
        onValueChange(value)
        if (importType.walletType == WalletType.View || importType.walletType == WalletType.PrivateKey) {
            viewModel.onInput(value.text, importType.chain)
        }
    }

    DisposableEffect(Unit) {
        viewModel.onResolved(onResolved)

        onDispose {}
    }

    val errorColor = MaterialTheme.colorScheme.error
    val clipboardManager = LocalClipboard.current.nativeClipboard
    val interactionSource = remember { MutableInteractionSource() }

    Column(
        modifier = Modifier.fillMaxSize()
    ) {
        Box(modifier = Modifier.fillMaxWidth()) {
            BasicTextField(
                modifier = Modifier.fillMaxWidth(),
                onValueChange = ::updateInput,
                value = inputState,
                textStyle = MaterialTheme.typography.bodyLarge.copy(
                    color = MaterialTheme.colorScheme.onSurface
                ),
                minLines = 2,
                cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                visualTransformation = {
                    if (importType.walletType == WalletType.View  || importType.walletType == WalletType.PrivateKey) {
                        return@BasicTextField TransformedText(it, OffsetMapping.Identity)
                    }
                    TransformedText(
                        highlightErrors(
                            it.text,
                            errorColor = errorColor
                        ),
                        OffsetMapping.Identity
                    )
                },
                decorationBox = { innerTextField ->
                    if (inputState.text.isEmpty()) {
                        Text(
                            text = when (importType.walletType) {
                                WalletType.View -> stringResource(R.string.wallet_import_address_field)
                                WalletType.PrivateKey -> stringResource(R.string.common_private_key)
                                else -> stringResource(R.string.common_secret_phrase)
                            },
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.secondary,
                        )
                    }
                    innerTextField()
                },
                keyboardOptions = KeyboardOptions(
                    keyboardType = KeyboardType.Password,
                    platformImeOptions = PlatformImeOptions("flagNoPersonalizedLearning"),
                    autoCorrectEnabled = false,
                ),
                interactionSource = interactionSource,
            )
            Row(
                modifier = Modifier.align(Alignment.TopEnd),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                if (uiState is NameRecordState.Loading) {
                    CircularProgressIndicator16()
                    Spacer(modifier = Modifier.size(8.dp))
                }
                if (uiState is NameRecordState.Complete) {
                    SelectionCheckmark(
                        color = MaterialTheme.colorScheme.tertiary,
                    )
                    Spacer(modifier = Modifier.size(8.dp))
                }
                if (uiState is NameRecordState.Error) {
                    Icon(
                        modifier = Modifier.size(24.dp),
                        imageVector = AppIcons.Error,
                        contentDescription = "Name is resolved",
                        tint = MaterialTheme.colorScheme.error,
                    )
                    Spacer(modifier = Modifier.size(8.dp))
                }
            }
        }
        Spacer16()
        Box(
            modifier = Modifier.fillMaxWidth(),
        ) {
            FieldBottomAction(
                modifier = Modifier
                    .align(Alignment.Center)
                    .testTag("paste"),
                imageVector = AppIcons.ContentPaste,
                text = stringResource(id = R.string.common_paste),
            ) {
                val newValue = clipboardManager.getPlainText() ?: ""
                val pastedText = if (importType.walletType == WalletType.View || importType.walletType == WalletType.PrivateKey) {
                    newValue.trim()
                } else {
                    "$newValue "
                }
                updateInput(
                    TextFieldValue(
                        text = pastedText,
                        selection = TextRange(pastedText.length),
                    )
                )
                if (shouldProtectInput(importType.walletType)) {
                    clipboardManager.clear()
                }
            }
        }
    }
}

private fun highlightErrors(text: String, errorColor: Color): AnnotatedString {
    val validateResult = GemValidatePhraseOperator().invoke(text)
    val error = validateResult.exceptionOrNull()
    val invalidWords = (error as? InvalidWords)?.words.orEmpty().filter { it.isNotBlank() }.toSet()

    return highlightInvalidPhraseWords(text, errorColor, invalidWords)
}

internal fun highlightInvalidPhraseWords(
    text: String,
    errorColor: Color,
    invalidWords: Set<String>,
): AnnotatedString {
    return buildAnnotatedString {
        append(text)
        if (invalidWords.isEmpty()) {
            return@buildAnnotatedString
        }
        text.wordRanges().forEach { range ->
            val word = text.substring(range)
            if (word in invalidWords) {
                addStyle(
                    style = SpanStyle(color = errorColor),
                    start = range.first,
                    end = range.last + 1,
                )
            }
        }
    }
}

private fun String.wordRanges(): Sequence<IntRange> = sequence {
    var start = -1
    for (index in indices) {
        if (this@wordRanges[index].isWhitespace()) {
            if (start != -1) {
                yield(start until index)
                start = -1
            }
        } else if (start == -1) {
            start = index
        }
    }
    if (start != -1) {
        yield(start until length)
    }
}
