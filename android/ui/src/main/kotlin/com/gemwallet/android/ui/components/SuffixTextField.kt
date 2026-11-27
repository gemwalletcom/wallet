package com.gemwallet.android.ui.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.TextFieldDecorator
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.placeCursorAtEnd
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.secondaryFaded

@Composable
fun SuffixTextField(
    value: String,
    onValueChange: (String) -> Unit,
    suffix: String,
    modifier: Modifier = Modifier,
    placeholder: String = "",
    focusRequester: FocusRequester? = null,
    keyboardOptions: KeyboardOptions = KeyboardOptions.Default,
    textAlign: TextAlign = TextAlign.End,
) {
    val state = rememberValueTextFieldState(value)
    Row(
        modifier = modifier,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BasicTextField(
            modifier = Modifier
                .weight(1f)
                .then(focusRequester?.let { Modifier.focusRequester(it) } ?: Modifier)
                .onFocusChanged { focus ->
                    if (focus.isFocused) {
                        state.edit { placeCursorAtEnd() }
                    }
                },
            state = state,
            inputTransformation = valueChangeTransformation(value, onValueChange),
            lineLimits = TextFieldLineLimits.SingleLine,
            textStyle = MaterialTheme.typography.bodyLarge.copy(
                color = MaterialTheme.colorScheme.onSurface,
                textAlign = textAlign,
            ),
            keyboardOptions = keyboardOptions,
            cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
            decorator = TextFieldDecorator { innerTextField ->
                Box(modifier = Modifier.fillMaxWidth(), contentAlignment = if (textAlign == TextAlign.Start) Alignment.CenterStart else Alignment.CenterEnd) {
                    if (state.text.isEmpty() && placeholder.isNotEmpty()) {
                        Text(
                            text = placeholder,
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.secondaryFaded,
                        )
                    }
                    innerTextField()
                }
            },
        )
        Spacer(modifier = Modifier.width(paddingHalfSmall))
        Text(
            text = suffix,
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.secondary,
        )
    }
}
