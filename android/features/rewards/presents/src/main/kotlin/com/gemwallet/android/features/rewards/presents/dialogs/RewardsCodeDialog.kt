package com.gemwallet.android.features.rewards.presents.dialogs

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.GemTextField
import com.gemwallet.android.ui.components.filters.FormDialog
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.Spacer8
import com.gemwallet.android.ui.theme.paddingDefault

@Composable
internal fun RewardsCodeDialog(isVisible: Boolean, title: String, label: String, text: String, isLoading: Boolean, onDone: (String) -> Unit, onDismiss: () -> Unit, info: String? = null) {
    var input by remember(isVisible, text) { mutableStateOf(text) }
    val context = LocalContext.current

    val focusRequester = remember { FocusRequester() }

    val doneAction: () -> Unit = done@{
        if (input.isEmpty() || isLoading) return@done
        context.requestAuth(AuthRequest.Default) { onDone(input) }
    }
    val done: @Composable () -> Unit = {
        TextButton(
            onClick = doneAction,
            enabled = input.isNotEmpty() && !isLoading,
        ) {
            if (isLoading) {
                CircularProgressIndicator16()
            } else {
                Text(stringResource(R.string.common_done))
            }
        }
    }

    FormDialog(
        isVisible = isVisible,
        title = title,
        onDismiss = onDismiss,
        doneAction = done,
    ) {
        LaunchedEffect(Unit) {
            focusRequester.requestFocus()
        }
        GemTextField(
            modifier = Modifier.fillMaxWidth().focusRequester(focusRequester),
            label = label,
            value = input,
            onValueChange = {
                input = it
            },
            singleLine = true,
            keyboardActions = KeyboardActions(onDone = { doneAction() }),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
        )
        info?.let {
            Spacer8()
            Text(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = paddingDefault),
                text = it,
                color = MaterialTheme.colorScheme.secondary,
                style = MaterialTheme.typography.bodyMedium,
                textAlign = TextAlign.Center,
            )
        }
        Spacer16()
    }
}
