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
internal fun CreateRewardsCodeDialog(isVisible: Boolean, isLoading: Boolean, onUsername: (String) -> Unit, onDismiss: () -> Unit) {
    var username by remember(isVisible) { mutableStateOf("") }
    val context = LocalContext.current

    val focusRequester = remember { FocusRequester() }

    val doneAction: () -> Unit = done@{
        if (username.isEmpty() || isLoading) return@done
        context.requestAuth(AuthRequest.Default) { onUsername(username) }
    }
    val done: @Composable () -> Unit = {
        TextButton(
            onClick = doneAction,
            enabled = username.isNotEmpty() && !isLoading,
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
        title = stringResource(R.string.rewards_create_referral_code_title),
        onDismiss = onDismiss,
        doneAction = done,
    ) {
        LaunchedEffect(Unit) {
            focusRequester.requestFocus()
        }
        GemTextField(
            modifier = Modifier.fillMaxWidth().focusRequester(focusRequester),
            label = stringResource(id = R.string.rewards_username),
            value = username,
            onValueChange = {
                username = it
            },
            singleLine = true,
            keyboardActions = KeyboardActions(onDone = { doneAction() }),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
        )
        Spacer8()
        Text(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = paddingDefault),
            text = stringResource(R.string.rewards_create_referral_code_info),
            color = MaterialTheme.colorScheme.secondary,
            style = MaterialTheme.typography.bodyMedium,
            textAlign = TextAlign.Companion.Center,
        )
        Spacer16()
    }
}
