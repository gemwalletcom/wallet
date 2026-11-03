package com.gemwallet.android.features.onboarding.presents.import_wallet.components

import android.view.inputmethod.EditorInfo
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.OutputTransformation
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.InterceptPlatformTextInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.PlatformTextInputInterceptor
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.input.KeyboardType
import com.gemwallet.android.features.onboarding.viewmodels.import_wallet.ImportInputUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.FieldBottomAction
import com.gemwallet.android.ui.components.clipboard.clear
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.components.clipboard.getPlainText
import com.gemwallet.android.ui.components.fields.NameResolveIndicator
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space8
import uniffi.gemstone.GemNameIndicator

private val noPersonalizedLearning = PlatformTextInputInterceptor { request, nextHandler ->
    nextHandler.startInputMethod { outAttributes ->
        val connection = request.createInputConnection(outAttributes)
        outAttributes.imeOptions = outAttributes.imeOptions or EditorInfo.IME_FLAG_NO_PERSONALIZED_LEARNING
        connection
    }
}

@Composable
internal fun ImportInput(state: TextFieldState, input: ImportInputUIModel, indicator: GemNameIndicator?, onScan: (() -> Unit)?) {
    val clipboardManager = LocalContext.current.clipboardManager()
    val interactionSource = remember { MutableInteractionSource() }
    val errorColor = MaterialTheme.colorScheme.error
    val highlight = remember(input, errorColor) { invalidWordsHighlight(input, errorColor) }

    Column(
        modifier = Modifier.fillMaxSize(),
    ) {
        Box(modifier = Modifier.fillMaxWidth()) {
            InterceptPlatformTextInput(noPersonalizedLearning) {
                BasicTextField(
                    modifier = Modifier
                        .fillMaxWidth()
                        .testTag("importInputField"),
                    state = state,
                    textStyle = MaterialTheme.typography.bodyLarge.copy(
                        color = MaterialTheme.colorScheme.onSurface,
                    ),
                    lineLimits = TextFieldLineLimits.MultiLine(minHeightInLines = 2),
                    cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                    outputTransformation = highlight,
                    decorator = { innerTextField ->
                        if (state.text.isEmpty()) {
                            Text(
                                text = stringResource(input.placeholder),
                                style = MaterialTheme.typography.bodyLarge,
                                color = MaterialTheme.colorScheme.secondary,
                            )
                        }
                        innerTextField()
                    },
                    keyboardOptions = KeyboardOptions(
                        keyboardType = KeyboardType.Password,
                        autoCorrectEnabled = false,
                    ),
                    interactionSource = interactionSource,
                )
            }
            Row(
                modifier = Modifier.align(Alignment.TopEnd),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                NameResolveIndicator(indicator)
                if (indicator != null) {
                    Spacer(modifier = Modifier.size(space8))
                }
            }
        }
        Spacer16()
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(paddingSmall, Alignment.CenterHorizontally),
        ) {
            FieldBottomAction(
                modifier = Modifier.testTag("paste"),
                imageVector = AppIcons.ContentPaste,
                text = stringResource(id = R.string.common_paste),
            ) {
                val pastedText = clipboardManager.getPlainText()?.trim() ?: return@FieldBottomAction
                state.setTextAndPlaceCursorAtEnd(pastedText)
                if (input.protectsInput) {
                    clipboardManager.clear()
                }
            }
            if (onScan != null) {
                FieldBottomAction(
                    modifier = Modifier.testTag("scan"),
                    imageVector = AppIcons.QrCodeScanner,
                    text = stringResource(id = R.string.wallet_scan),
                    onClick = onScan,
                )
            }
        }
    }
}

private fun invalidWordsHighlight(input: ImportInputUIModel, color: Color) = OutputTransformation {
    input.invalidWordRanges(asCharSequence()).forEach { addStyle(SpanStyle(color = color), it.first, it.last + 1) }
}
