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
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PlatformImeOptions
import androidx.compose.ui.text.input.TextFieldValue
import com.gemwallet.android.features.import_wallet.viewmodels.ImportInputUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.FieldBottomAction
import com.gemwallet.android.ui.components.clipboard.clear
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.components.clipboard.getPlainText
import com.gemwallet.android.ui.components.fields.NameResolveIndicator
import com.gemwallet.android.ui.components.fields.NameResolveIndicatorUIModel
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.space8

@Composable
internal fun ImportInput(inputState: TextFieldValue, input: ImportInputUIModel, indicator: NameResolveIndicatorUIModel?, onValueChange: (TextFieldValue) -> Unit) {
    val clipboardManager = LocalContext.current.clipboardManager()
    val interactionSource = remember { MutableInteractionSource() }

    Column(
        modifier = Modifier.fillMaxSize(),
    ) {
        Box(modifier = Modifier.fillMaxWidth()) {
            BasicTextField(
                modifier = Modifier.fillMaxWidth(),
                onValueChange = onValueChange,
                value = inputState,
                textStyle = MaterialTheme.typography.bodyLarge.copy(
                    color = MaterialTheme.colorScheme.onSurface,
                ),
                minLines = 2,
                cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                decorationBox = { innerTextField ->
                    if (inputState.text.isEmpty()) {
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
                    platformImeOptions = PlatformImeOptions("flagNoPersonalizedLearning"),
                    autoCorrectEnabled = false,
                ),
                interactionSource = interactionSource,
            )
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
                val pastedText = newValue.trim()
                onValueChange(
                    TextFieldValue(
                        text = pastedText,
                        selection = TextRange(pastedText.length),
                    ),
                )
                if (input.protectsInput) {
                    clipboardManager.clear()
                }
            }
        }
    }
}
